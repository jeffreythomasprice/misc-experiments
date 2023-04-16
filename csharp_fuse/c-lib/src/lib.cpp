#include "lib.h"

#include <sstream>

#include "Logger.h"

struct _FuseData {
	std::string mountPoint;
	fuse* f;
	fuse_chan* chan;
};

Logger createLogger(LogFunc trace, LogFunc debug, LogFunc information,
					LogFunc warning, LogFunc error, LogFunc critical) {
	return new _Logger(trace, debug, information, warning, error, critical);
}

void freeLogger(Logger logger) {
	auto _logger = static_cast<_Logger*>(logger);
	delete _logger;
}

struct stat* createStat() { return new struct stat; }

void freeStat(struct stat* s) { delete s; }

int mountAndRun(Logger logger, int argc, const char** argv,
				fuse_operations* ops, void (*callback)(FuseData)) {
	auto _logger = static_cast<_Logger*>(logger);

	fuse_args args = FUSE_ARGS_INIT(argc, (char**)argv);
	char* mountPoint;
	int multithreaded;
	int foreground;
	fuse_parse_cmdline(&args, &mountPoint, &multithreaded, &foreground);

	auto chan = fuse_mount(mountPoint, &args);
	{
		std::stringstream ss;
		ss << "mounted " << mountPoint;
		_logger->logTrace(ss.str());
	}

	auto f = fuse_new(chan, &args, ops, sizeof(fuse_operations), NULL);

	auto data = new _FuseData{.mountPoint = mountPoint, .f = f, .chan = chan};
	callback((FuseData)data);

	auto result = fuse_loop(f);
	{
		std::stringstream ss;
		ss << "fuse exited " << result;
		_logger->logTrace(ss.str());
	}
	delete data;
	return result;
}

void unmountAndExit(Logger logger, FuseData data) {
	auto _logger = static_cast<_Logger*>(logger);
	auto _data = static_cast<_FuseData*>(data);

	{
		std::stringstream ss;
		ss << "unmounting " << _data->mountPoint;
		_logger->logTrace(ss.str());
	}
	fuse_unmount(_data->mountPoint.c_str(), _data->chan);

	_logger->logTrace("exiting fuse");
	fuse_exit(_data->f);
}
