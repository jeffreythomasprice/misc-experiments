#pragma once

#include <iostream>
#include <optional>
#include <sstream>
#include <string>

#include "common.h"

enum class LogLevel {
	FATAL = 1,
	ERROR = 2,
	WARN = 3,
	INFO = 4,
	DEBUG = 5,
	TRACE = 6,
};

struct LogMessage {
	uint64_t timestamp;
	LogLevel level;
	std::string message;
};

class LogOStream : public std::ostream {
   private:
	class LogStringBuf : public std::stringbuf {
	   public:
		LogLevel level;

	   public:
		LogStringBuf(LogLevel level);

		virtual int sync();
	};

	LogStringBuf buf;

   public:
	LogOStream(LogLevel level);
	~LogOStream();
};

void initLogging(size_t maxBufferSizeBeforeFlush,
				 std::optional<Napi::ThreadSafeFunction> emitLogCallback);
void deinitLogging();
void unbufferLogs();
void log(uint64_t timestamp, LogLevel level, const std::string& message);
void log(LogLevel level, const std::string& message);
LogOStream fatal();
LogOStream error();
LogOStream warn();
LogOStream info();
LogOStream debug();
LogOStream trace();

std::ostream& operator<<(std::ostream& s, LogLevel level);
std::ostream& operator<<(std::ostream& s, const LogMessage& logMessage);
