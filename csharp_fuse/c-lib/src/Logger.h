#pragma once

#include <string>

#include "lib.h"

class _Logger {
   private:
	LogFunc _trace;
	LogFunc _debug;
	LogFunc _information;
	LogFunc _warning;
	LogFunc _error;
	LogFunc _critical;

   public:
	_Logger(LogFunc trace, LogFunc debug, LogFunc information, LogFunc warning,
			LogFunc error, LogFunc critical);

	void logTrace(const std::string& s);
	void logDebug(const std::string& s);
	void logInformation(const std::string& s);
	void logWarning(const std::string& s);
	void logError(const std::string& s);
	void logCritical(const std::string& s);
};