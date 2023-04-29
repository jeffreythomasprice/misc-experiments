#include "fuse-utils.h"

#include "logging.h"
#include "thread-utils.h"

Napi::Value fuseConnInfoToJSObject(const Napi::Env& env, const fuse_conn_info* conn) {
	auto result = Napi::Object::New(env);
	result.Set("proto_major", Napi::Number::New(env, conn->proto_major));
	result.Set("proto_minor", Napi::Number::New(env, conn->proto_minor));
	result.Set("async_read", Napi::Number::New(env, conn->async_read));
	result.Set("max_write", Napi::Number::New(env, conn->max_write));
	result.Set("max_readahead", Napi::Number::New(env, conn->max_readahead));
	result.Set("capable", Napi::Number::New(env, conn->capable));
	result.Set("want", Napi::Number::New(env, conn->want));
	result.Set("max_background", Napi::Number::New(env, conn->max_background));
	result.Set("congestion_threshold", Napi::Number::New(env, conn->congestion_threshold));
	result.Freeze();
	return result;
}

void jsObjectToStat(const Napi::Object& value, struct stat* stat) {
	auto getInt32 = [&value](const std::string& name) {
		return value.Get(name).As<Napi::Number>().Int32Value();
	};

	auto getTimespec = [&value](const std::string& name) {
		auto actualValue = value.Get(name).As<Napi::Object>();
		timespec result;
		result.tv_sec = actualValue.Get("tv_sec").As<Napi::Number>();
		result.tv_nsec = actualValue.Get("tv_nsec").As<Napi::Number>();
		return result;
	};

	stat->st_dev = getInt32("st_dev");
	stat->st_ino = getInt32("st_ino");
	stat->st_nlink = getInt32("st_nlink");
	stat->st_mode = getInt32("st_mode");
	stat->st_uid = getInt32("st_uid");
	stat->st_gid = getInt32("st_gid");
	stat->st_rdev = getInt32("st_rdev");
	stat->st_size = getInt32("st_size");
	stat->st_blksize = getInt32("st_blksize");
	stat->st_blocks = getInt32("st_blocks");
	stat->st_atim = getTimespec("st_atim");
	stat->st_mtim = getTimespec("st_mtim");
	stat->st_ctim = getTimespec("st_ctim");
}

void jsArrayToReaddirResults(const Napi::Array& results, void* buf, fuse_fill_dir_t filler) {
	for (size_t i = 0; i < results.Length(); i++) {
		auto dir = results.Get(i).As<Napi::Object>();
		std::string path = dir.Get("path").As<Napi::String>();
		trace() << "jsArrayToReaddirResults, path = " << path;
		if (dir.Has("stat")) {
			struct stat stat;
			jsObjectToStat(dir.Get("stat").As<Napi::Object>(), &stat);
			filler(buf, path.c_str(), &stat, 0);
		} else {
			filler(buf, path.c_str(), nullptr, 0);
		}
	}
}

FuseUserData::FuseUserData(const Napi::Env& env, const Napi::Object& callbacks)
	: destroyed(false) {
	auto getCallback = [&callbacks, &env](const std::string& name) -> std::optional<Napi::ThreadSafeFunction> {
		if (callbacks.Has(name)) {
			std::stringstream functionName;
			functionName << name << " callback";
			return Napi::ThreadSafeFunction::New(
				env,
				callbacks.Get(name).As<Napi::Function>(),
				functionName.str(),
				// max queue size, 0 = unlimited
				0,
				// initial thread count
				1
			);
		} else {
			return std::nullopt;
		}
	};

	initCallback = getCallback("init");
	destroyCallback = getCallback("destroy");
	getattrCallback = getCallback("getattr");
	readdirCallback = getCallback("readdir");
	// TODO other callbacks
}

FuseUserData::~FuseUserData() {
	destroy();

	auto releaseCallback = [](const std::optional<Napi::ThreadSafeFunction>& c) {
		if (c.has_value()) {
			c.value().Release();
		}
	};

	releaseCallback(initCallback);
	releaseCallback(destroyCallback);
	releaseCallback(getattrCallback);
	releaseCallback(readdirCallback);
}

void FuseUserData::init(fuse_conn_info* connectionInfo) {
	const auto methodName = "FuseUserData::init";
	trace() << methodName << " begin";
	if (initCallback.has_value()) {
		trace() << methodName << " invoking callback";
		await(initCallback.value(), [connectionInfo](const Napi::Env& env, Napi::Function f) {
			auto jsConnectionInfo = fuseConnInfoToJSObject(env, connectionInfo);
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
			await(destroyCallback.value(), [](const Napi::Env& env, Napi::Function f) {
				return f({});
			});
		} else {
			trace() << methodName << " no callback provided";
		}
	}
	trace() << methodName << " end";
}

int FuseUserData::getattr(const std::string& path, struct stat* stat) {
	const auto methodName = "FuseUserData::getattr";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (getattrCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			getattrCallback.value(),
			[&path](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path)});
			},
			[stat](const Napi::Value& value) {
				if (value.IsNumber()) {
					return value.As<Napi::Number>().Int32Value();
				} else if (value.IsObject()) {
					jsObjectToStat(value.As<Napi::Object>(), stat);
					return 0;
				} else {
					throw new std::logic_error("expected either number or object");
				}
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::readdir(const std::string& path, void* buf, fuse_fill_dir_t filler) {
	const auto methodName = "FuseUserData::readdir";
	trace() << methodName << " begin";
	int result = -ENOENT;
	if (readdirCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			readdirCallback.value(),
			[&path](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path)});
			},
			[&buf, &filler](const Napi::Value& value) {
				if (value.IsNumber()) {
					return value.As<Napi::Number>().Int32Value();
				} else if (value.IsArray()) {
					jsArrayToReaddirResults(value.As<Napi::Array>(), buf, filler);
					return 0;
				} else {
					throw new std::logic_error("expected either number or object");
				}
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end";
	return result;
}