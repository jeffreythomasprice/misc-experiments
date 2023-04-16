#pragma once

#define FUSE_USE_VERSION 35
#include <fuse.h>

extern "C" {

typedef void (*LogFunc)(const char* message);

typedef void* Logger;

typedef void* FuseData;

Logger createLogger(LogFunc trace, LogFunc debug, LogFunc information,
					LogFunc warning, LogFunc error, LogFunc critical);
void freeLogger(Logger logger);

struct stat* createStat();
void freeStat(struct stat* s);

int mountAndRun(Logger logger, int argc, const char** argv,
				fuse_operations* ops, void (*callback)(FuseData));
void unmountAndExit(Logger logger, FuseData data);
}