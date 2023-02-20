#include "App.h"
#include "AppState.h"

// TODO clean includes

#include <iostream>
#include <chrono>
#include <optional>
#include <cmath>
#include <sstream>
#include <thread>
#include <future>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/val.h>
#include <emscripten/fetch.h>
#endif

#include <GL/gl.h>

enum class LogLevel
{
	Verbose = 0,
	Debug,
	Info,
	Warning,
	Error,
};

std::ostream &operator<<(std::ostream &s, LogLevel level)
{
	switch (level)
	{
	case LogLevel::Verbose:
		return s << "Verbose";
	case LogLevel::Debug:
		return s << "Debug";
	case LogLevel::Info:
		return s << "Info";
	case LogLevel::Warning:
		return s << "Warning";
	case LogLevel::Error:
		return s << "Error";
	default:
		return s << "LogLevel(" << (int)level << ")";
	}
}

class Logger
{
private:
	LogLevel level;

public:
	Logger();
	virtual ~Logger();

	LogLevel getLevel() const;
	void setLevel(LogLevel level);

	bool isEnabled(LogLevel level) const;

	void verbose(const std::string &s);
	void debug(const std::string &s);
	void info(const std::string &s);
	void warning(const std::string &s);
	void error(const std::string &s);

	virtual void log(LogLevel level, const std::string &s) = 0;

protected:
	const std::string formatLogLine(LogLevel level, const std::string &s) const;
};

Logger::Logger() : level(LogLevel::Debug) {}

Logger::~Logger() {}

LogLevel Logger::getLevel() const { return level; }

void Logger::setLevel(LogLevel level) { this->level = level; }

bool Logger::isEnabled(LogLevel level) const
{
	return level >= this->level;
}

void Logger::verbose(const std::string &s) { log(LogLevel::Verbose, s); }

void Logger::debug(const std::string &s) { log(LogLevel::Debug, s); }

void Logger::info(const std::string &s) { log(LogLevel::Info, s); }

void Logger::warning(const std::string &s) { log(LogLevel::Warning, s); }

void Logger::error(const std::string &s) { log(LogLevel::Error, s); }

const std::string Logger::formatLogLine(LogLevel level, const std::string &s) const
{
	std::stringstream ss;
	// TODO timestamp
	ss << level << ": " << s;
	return ss.str();
}

class ConsoleLogger : public Logger
{
public:
	virtual ~ConsoleLogger();

	virtual void log(LogLevel level, const std::string &s) override;
};

ConsoleLogger::~ConsoleLogger() {}

void ConsoleLogger::log(LogLevel level, const std::string &s)
{
	if (isEnabled(level))
	{
		auto formatted = formatLogLine(level, s);
		switch (level)
		{
		case LogLevel::Warning:
			emscripten_console_warn(formatted.c_str());
			break;
		case LogLevel::Error:
			emscripten_console_error(formatted.c_str());
			break;
		default:
			emscripten_console_log(formatted.c_str());
			break;
		}
	}
}

// TODO delete me?
std::string
emscriptenResultToString(int result)
{
	switch (result)
	{
	case EMSCRIPTEN_RESULT_SUCCESS:
		return "EMSCRIPTEN_RESULT_SUCCESS";
	case EMSCRIPTEN_RESULT_DEFERRED:
		return "EMSCRIPTEN_RESULT_DEFERRED";
	case EMSCRIPTEN_RESULT_NOT_SUPPORTED:
		return "EMSCRIPTEN_RESULT_NOT_SUPPORTED";
	case EMSCRIPTEN_RESULT_FAILED_NOT_DEFERRED:
		return "EMSCRIPTEN_RESULT_FAILED_NOT_DEFERRED";
	case EMSCRIPTEN_RESULT_INVALID_TARGET:
		return "EMSCRIPTEN_RESULT_INVALID_TARGET";
	case EMSCRIPTEN_RESULT_UNKNOWN_TARGET:
		return "EMSCRIPTEN_RESULT_UNKNOWN_TARGET";
	case EMSCRIPTEN_RESULT_INVALID_PARAM:
		return "EMSCRIPTEN_RESULT_INVALID_PARAM";
	case EMSCRIPTEN_RESULT_FAILED:
		return "EMSCRIPTEN_RESULT_FAILED";
	case EMSCRIPTEN_RESULT_NO_DATA:
		return "EMSCRIPTEN_RESULT_NO_DATA";
	case EMSCRIPTEN_RESULT_TIMED_OUT:
		return "EMSCRIPTEN_RESULT_TIMED_OUT";
	default:
		std::stringstream ss;
		ss << "unknown EMSCRIPTEN_RESULT_ enum " << result;
		return ss.str();
	}
}

// TODO move me
class DownloadManager
{
public:
	typedef std::function<void(const std::string &)> OnSuccessCallback;
	typedef std::function<void(unsigned short, const std::string &)> OnErrorCallback;
	typedef std::function<void(uint64_t, uint64_t)> OnProgressCallback;

private:
	struct Callbacks
	{
		emscripten_fetch_t *fetch;
		OnSuccessCallback onSuccess;
		OnErrorCallback onError;
		OnProgressCallback onProgress;
	};

	std::vector<std::shared_ptr<Callbacks>> callbacks;

public:
	void makeGetRequest(
		const std::string &url,
		OnSuccessCallback onSuccess,
		OnErrorCallback onError = nullptr,
		OnProgressCallback onProgress = nullptr);
	std::future<std::string> makeGetRequest(const std::string &url);

private:
	static void fetchOnSuccess(emscripten_fetch_t *fetch);
	static void fetchOnError(emscripten_fetch_t *fetch);
	static void fetchOnProgress(emscripten_fetch_t *fetch);
	static std::shared_ptr<Callbacks> getCallbacks(emscripten_fetch_t *fetch);
	static void removeCallbacks(std::shared_ptr<DownloadManager::Callbacks> callbacks);
};

void DownloadManager::makeGetRequest(
	const std::string &url,
	OnSuccessCallback onSuccess,
	OnErrorCallback onError,
	OnProgressCallback onProgress)
{
	emscripten_fetch_attr_t attributes;
	emscripten_fetch_attr_init(&attributes);
	strcpy(attributes.requestMethod, "GET");
	attributes.attributes = EMSCRIPTEN_FETCH_LOAD_TO_MEMORY;
	attributes.onsuccess = fetchOnSuccess;
	attributes.onerror = fetchOnError;
	attributes.onprogress = fetchOnProgress;
	attributes.userData = this;

	// TODO support a timeout

	auto c = std::make_shared<Callbacks>();
	// TODO lock?
	callbacks.push_back(c);
	c->fetch = emscripten_fetch(&attributes, url.c_str());
	c->onSuccess = onSuccess;
	c->onError = onError;
	c->onProgress = onProgress;
}

std::future<std::string> DownloadManager::makeGetRequest(const std::string &url)
{
	auto p = std::make_shared<std::promise<std::string>>();
	makeGetRequest(
		url,
		[p](const std::string &data)
		{
			p->set_value(data);
		},
		[p](unsigned short httpStatusCode, const std::string &httpStatusMessage)
		{
			// TODO custom exception class for status code and msg
			p->set_exception(std::make_exception_ptr(std::logic_error("TODO JEFF error msg")));
		});
	return p->get_future();
}

void DownloadManager::fetchOnSuccess(emscripten_fetch_t *fetch)
{
	auto callbacks = getCallbacks(fetch);
	if (callbacks && callbacks->onSuccess)
	{
		removeCallbacks(callbacks);
		callbacks->onSuccess(std::string(fetch->data, fetch->data + fetch->numBytes));
	}
}

void DownloadManager::fetchOnError(emscripten_fetch_t *fetch)
{
	auto callbacks = getCallbacks(fetch);
	if (callbacks && callbacks->onError)
	{
		removeCallbacks(callbacks);
		callbacks->onError(fetch->status, fetch->statusText);
	}
}

void DownloadManager::fetchOnProgress(emscripten_fetch_t *fetch)
{
	auto callbacks = getCallbacks(fetch);
	if (callbacks && callbacks->onProgress)
	{
		callbacks->onProgress(fetch->numBytes, fetch->totalBytes);
	}
}

std::shared_ptr<DownloadManager::Callbacks> DownloadManager::getCallbacks(emscripten_fetch_t *fetch)
{
	auto manager = (DownloadManager *)fetch->userData;
	// TODO lock?
	for (auto &c : manager->callbacks)
	{
		if (c->fetch == fetch)
		{
			return c;
		}
	}
	return nullptr;
}

void DownloadManager::removeCallbacks(std::shared_ptr<DownloadManager::Callbacks> callbacks)
{
	auto manager = (DownloadManager *)callbacks->fetch->userData;
	// TODO lock?
	for (auto i = manager->callbacks.begin(); i != manager->callbacks.end(); i++)
	{
		if (*i == callbacks)
		{
			manager->callbacks.erase(i);
			return;
		}
	}
}

class DemoState : public AppState
{
private:
	std::shared_ptr<Logger> logger;
	DownloadManager downloadManager;

public:
	DemoState(std::shared_ptr<Logger> logger) : logger(logger) {}

	void activate() override
	{
		// downloadManager.makeGetRequest(
		// 	"index.html",
		// 	[](const std::string &data)
		// 	{
		// 		emscripten_console_log(data.c_str());
		// 		// std::cout << "TODO JEFF success " << data << std::endl;
		// 	},
		// 	[](unsigned short httpStatusCode, const std::string &httpStatusMessage)
		// 	{
		// 		std::cout << "TODO JEFF error " << httpStatusCode << ", " << httpStatusMessage << std::endl;
		// 	},
		// 	[](uint64_t numBytes, uint64_t totalBytes)
		// 	{
		// 		std::cout << "TODO JEFF progress " << numBytes << " " << totalBytes << std::endl;
		// 	});
		try
		{
			// TODO blocks main thread
			auto result = downloadManager.makeGetRequest("index.html").get();
			std::stringstream ss;
			ss << "result from get request:\n"
			   << result;
			logger->debug(ss.str());
		}
		catch (const std::exception &e)
		{
			std::stringstream ss;
			ss << "error making request: " << e.what();
			logger->error(ss.str());
		}
	}

	void resize(int width, int height) override
	{
		glViewport(0, 0, width, height);
	}

	void render() override
	{
		glClearColor(0.25f, 0.5f, 0.75f, 1.0f);
		glClear(GL_COLOR_BUFFER_BIT);
	}

	std::shared_ptr<AppState> update(const std::chrono::milliseconds &d) override
	{
		// std::cout << "DemoState update " << d.count() << "ms" << std::endl;
		return nullptr;
	}
};

int main()
{
	auto logger = std::make_shared<ConsoleLogger>();
	logger->setLevel(LogLevel::Verbose);
	logger->verbose("test");
	logger->debug("test");
	logger->info("test");
	logger->warning("test");
	logger->error("test");
	App app(std::make_shared<DemoState>(logger));
	return 0;
}