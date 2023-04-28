#include "fuse-utils.h"

#include "logging.h"
#include "thread-utils.h"

Napi::Value fuseConnInfoToObject(const Napi::Env& env,
								 const fuse_conn_info* conn) {
	auto result = Napi::Object::New(env);
	result.Set("proto_major", Napi::Number::New(env, conn->proto_major));
	result.Set("proto_minor", Napi::Number::New(env, conn->proto_minor));
	result.Set("async_read", Napi::Number::New(env, conn->async_read));
	result.Set("max_write", Napi::Number::New(env, conn->max_write));
	result.Set("max_readahead", Napi::Number::New(env, conn->max_readahead));
	result.Set("capable", Napi::Number::New(env, conn->capable));
	result.Set("want", Napi::Number::New(env, conn->want));
	result.Set("max_background", Napi::Number::New(env, conn->max_background));
	result.Set("congestion_threshold",
			   Napi::Number::New(env, conn->congestion_threshold));
	result.Freeze();
	return result;
}

FuseUserData::FuseUserData(const Napi::Env& env, const Napi::Object& callbacks)
	: destroyed(false) {
	if (callbacks.Has("init")) {
		initCallback = Napi::ThreadSafeFunction::New(
			env, callbacks.Get("init").As<Napi::Function>(), "init callback",
			// max queue size, 0 = unlimited
			0,
			// initial thread count
			1);
	}

	if (callbacks.Has("destroy")) {
		destroyCallback = Napi::ThreadSafeFunction::New(
			env, callbacks.Get("destroy").As<Napi::Function>(),
			"destroy callback",
			// max queue size, 0 = unlimited
			0,
			// initial thread count
			1);
	}

	// TODO other callbacks
}

FuseUserData::~FuseUserData() {
	destroy();

	if (initCallback.has_value()) {
		initCallback.value().Release();
	}
	if (destroyCallback.has_value()) {
		destroyCallback.value().Release();
	}
}

void FuseUserData::init(fuse_conn_info* connectionInfo) {
	const auto methodName = "FuseUserData::init";
	trace() << methodName << " begin";
	if (initCallback.has_value()) {
		trace() << methodName << " invoking callback";
		await(initCallback.value(), [connectionInfo](const Napi::Env& env,
													 Napi::Function f) {
			auto jsConnectionInfo = fuseConnInfoToObject(env, connectionInfo);
			return f({jsConnectionInfo});
		});
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end";
}

void FuseUserData::destroy() {
	const auto methodName = "FuseUserData::destroy";
	trace() << methodName << " begin";
	if (destroyed) {
		trace() << methodName << " already destroyed";
	} else {
		destroyed = true;
		if (destroyCallback.has_value()) {
			trace() << methodName << " invoking callback";
			destroyCallback.value().BlockingCall(
				(void*)nullptr,
				[](const Napi::Env& env, Napi::Function f, void*) {
					// TODO handle promise
					auto promise = f({});
				});
		} else {
			trace() << methodName << " no callback provided";
		}
	}
	trace() << methodName << " end";
}