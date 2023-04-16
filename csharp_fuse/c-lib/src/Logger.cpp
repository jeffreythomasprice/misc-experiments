#include "Logger.h"

_Logger::_Logger(LogFunc trace, LogFunc debug, LogFunc information,
				 LogFunc warning, LogFunc error, LogFunc critical)
	: _trace(trace),
	  _debug(debug),
	  _information(information),
	  _warning(warning),
	  _error(error),
	  _critical(critical) {}

void _Logger::logTrace(const std::string& s) { _trace(s.c_str()); }

void _Logger::logDebug(const std::string& s) { _debug(s.c_str()); }

void _Logger::logInformation(const std::string& s) { _information(s.c_str()); }

void _Logger::logWarning(const std::string& s) { _warning(s.c_str()); }

void _Logger::logError(const std::string& s) { _error(s.c_str()); }

void _Logger::logCritical(const std::string& s) { _critical(s.c_str()); }