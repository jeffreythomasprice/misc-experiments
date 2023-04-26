#pragma once

#include <optional>

#include "common.h"

class FuseUserData {
   private:
	std::optional<Napi::ThreadSafeFunction> initCallback;
	std::optional<Napi::ThreadSafeFunction> destroyCallback;

	bool destroyed;

   public:
	FuseUserData(const Napi::Env& env, const Napi::Object& callbacks);
	~FuseUserData();

	void init(fuse_conn_info* connectionInfo);
	void destroy();
};
