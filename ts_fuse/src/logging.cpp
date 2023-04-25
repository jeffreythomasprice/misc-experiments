#include "logging.h"

#include <thread>
#include <vector>

std::mutex logBufferMutex;
std::vector<LogMessage> logBuffer;

size_t maxBufferSizeBeforeFlush;
std::optional<Napi::ThreadSafeFunction> emitLogCallback;

LogOStream::LogStringBuf::LogStringBuf(LogLevel level) : level(level) {}

int LogOStream::LogStringBuf::sync() {
	auto output = str();

	// reset back to empty
	str("");

	if (output.empty()) {
		return 0;
	}

	// trim last newline
	if (output[output.size() - 1] == '\n') {
		output = output.substr(0, output.size() - 1);
	}

	log(level, output);

	return output.size();
}

LogOStream::LogOStream(LogLevel level) : std::ostream(&buf), buf(level) {}

LogOStream::~LogOStream() { flush(); }

void initLogging(size_t _maxBufferSizeBeforeFlush,
				 std::optional<Napi::ThreadSafeFunction> _emitLogCallback) {
	maxBufferSizeBeforeFlush = _maxBufferSizeBeforeFlush;
	emitLogCallback = _emitLogCallback;
}

void deinitLogging() {
	maxBufferSizeBeforeFlush = 0;
	emitLogCallback = std::nullopt;
}

void unbufferLogs() {
	std::unique_lock lock(logBufferMutex);
	if (emitLogCallback.has_value()) {
		for (auto& logMessage : logBuffer) {
			emitLogCallback.value().BlockingCall(
				(void*)nullptr,
				[logMessage](Napi::Env env, Napi::Function f, void*) {
					f({Napi::Number::From(env, logMessage.timestamp),
					   Napi::Number::From(env, (int)logMessage.level),
					   Napi::String::From(env, logMessage.message)});
				});
		}
	}
	logBuffer.clear();
}

void log(uint64_t timestamp, LogLevel level, const std::string& message) {
	auto shouldFlush = false;
	{
		std::unique_lock lock(logBufferMutex);
		logBuffer.push_back({timestamp, level, message});
		if (logBuffer.size() >= maxBufferSizeBeforeFlush) {
			shouldFlush = true;
		}
	}

	if (shouldFlush) {
		unbufferLogs();
	}
}

void log(LogLevel level, const std::string& message) {
	struct timeval tv;
	gettimeofday(&tv, NULL);
	uint64_t now = ((uint64_t)tv.tv_sec) * 1000 + ((uint64_t)tv.tv_usec) / 1000;
	log(now, level, message);
}

LogOStream fatal() { return LogOStream(LogLevel::FATAL); }

LogOStream error() { return LogOStream(LogLevel::ERROR); }

LogOStream warn() { return LogOStream(LogLevel::WARN); }

LogOStream info() { return LogOStream(LogLevel::INFO); }

LogOStream debug() { return LogOStream(LogLevel::DEBUG); }

LogOStream trace() { return LogOStream(LogLevel::TRACE); }

std::ostream& operator<<(std::ostream& s, LogLevel level) {
	switch (level) {
		case LogLevel::FATAL:
			return s << "FATAL";
		case LogLevel::ERROR:
			return s << "ERROR";
		case LogLevel::WARN:
			return s << "WARN";
		case LogLevel::INFO:
			return s << "INFO";
		case LogLevel::DEBUG:
			return s << "DEBUG";
		case LogLevel::TRACE:
			return s << "TRACE";
		default:
			return s << "LogLevel(" << (int)level << ")";
	}
}

std::ostream& operator<<(std::ostream& s, const LogMessage& logMessage) {
	return s << "timestamp=" << logMessage.timestamp
			 << ", level=" << logMessage.level
			 << ", message=" << logMessage.message;
}
