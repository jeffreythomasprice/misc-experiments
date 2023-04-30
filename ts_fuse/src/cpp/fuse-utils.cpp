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
	auto getNum = [&value](const std::string& name) {
		return value.Get(name).As<Napi::Number>();
	};

	auto getTimespec = [&value](const std::string& name) {
		auto actualValue = value.Get(name).As<Napi::Object>();
		timespec result;
		result.tv_sec = actualValue.Get("tv_sec").As<Napi::Number>();
		result.tv_nsec = actualValue.Get("tv_nsec").As<Napi::Number>();
		return result;
	};

	stat->st_dev = getNum("st_dev").Uint32Value();
	stat->st_ino = getNum("st_ino").Uint32Value();
	stat->st_nlink = getNum("st_nlink").Uint32Value();
	stat->st_mode = getNum("st_mode");
	stat->st_uid = getNum("st_uid");
	stat->st_gid = getNum("st_gid");
	stat->st_rdev = getNum("st_rdev").Uint32Value();
	stat->st_size = getNum("st_size");
	stat->st_blksize = getNum("st_blksize");
	stat->st_blocks = getNum("st_blocks");
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

Napi::Value fuseFileInfoToJSObject(const Napi::Env& env, struct fuse_file_info* fileInfo) {
	auto result = Napi::Object::New(env);
	result.Set("flags", Napi::Number::New(env, fileInfo->flags));
	result.Set("writepage", Napi::Number::New(env, fileInfo->writepage));
	result.Set("direct_io", Napi::Boolean::New(env, fileInfo->direct_io));
	result.Set("keep_cache", Napi::Boolean::New(env, fileInfo->keep_cache));
	result.Set("flush", Napi::Boolean::New(env, fileInfo->flush));
	result.Set("nonseekable", Napi::Boolean::New(env, fileInfo->nonseekable));
	result.Set("flock_release", Napi::Boolean::New(env, fileInfo->flock_release));
	result.Set("fh", Napi::Number::New(env, fileInfo->fh));
	result.Set("lock_owner", Napi::Number::New(env, fileInfo->lock_owner));
	result.Freeze();
	return result;
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
	createCallback = getCallback("create");
	openCallback = getCallback("open");
	readCallback = getCallback("read");
	writeCallback = getCallback("write");
	unlinkCallback = getCallback("unlink");
	chmodCallback = getCallback("chmod");
	chownCallback = getCallback("chown");
	releaseCallback = getCallback("release");
}

FuseUserData::~FuseUserData() {
	destroy();

	auto release = [](const std::optional<Napi::ThreadSafeFunction>& c) {
		if (c.has_value()) {
			c.value().Release();
		}
	};

	release(initCallback);
	release(destroyCallback);
	release(getattrCallback);
	release(readdirCallback);
	release(createCallback);
	release(openCallback);
	release(readCallback);
	release(writeCallback);
	release(unlinkCallback);
	release(chmodCallback);
	release(chownCallback);
	release(releaseCallback);
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
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::create(const std::string& path, mode_t mode, struct fuse_file_info* fileInfo) {
	const auto methodName = "FuseUserData::create";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (createCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			createCallback.value(),
			[&path, mode, fileInfo](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path), Napi::Number::New(env, mode), fuseFileInfoToJSObject(env, fileInfo)});
			},
			[fileInfo](const Napi::Value& value) {
				if (value.IsNumber()) {
					return value.As<Napi::Number>().Int32Value();
				} else if (value.IsObject()) {
					fileInfo->fh = value.As<Napi::Object>().Get("fh").As<Napi::Number>().Int64Value();
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

int FuseUserData::open(const std::string& path, struct fuse_file_info* fileInfo) {
	const auto methodName = "FuseUserData::open";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (openCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			openCallback.value(),
			[&path, fileInfo](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path), fuseFileInfoToJSObject(env, fileInfo)});
			},
			[fileInfo](const Napi::Value& value) {
				if (value.IsNumber()) {
					return value.As<Napi::Number>().Int32Value();
				} else if (value.IsObject()) {
					fileInfo->fh = value.As<Napi::Object>().Get("fh").As<Napi::Number>().Int64Value();
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

int FuseUserData::read(const std::string& path, char* buf, size_t size, off_t offset, struct fuse_file_info* fileInfo) {
	const auto methodName = "FuseUserData::read";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (readCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			readCallback.value(),
			[&path, buf, size, offset, fileInfo](const Napi::Env& env, Napi::Function f) {
				return f(
					{Napi::String::From(env, path),
					 Napi::Buffer<uint8_t>::New(env, (uint8_t*)(buf + offset), (size_t)(size - offset)),
					 fuseFileInfoToJSObject(env, fileInfo)}
				);
			},
			[](const Napi::Value& value) {
				return value.As<Napi::Number>().Int32Value();
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::write(const std::string& path, const char* buf, size_t size, off_t offset, struct fuse_file_info* fileInfo) {
	const auto methodName = "FuseUserData::write";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (writeCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			writeCallback.value(),
			[&path, buf, size, offset, fileInfo](const Napi::Env& env, Napi::Function f) {
				return f(
					{Napi::String::From(env, path),
					 Napi::Buffer<uint8_t>::New(env, (uint8_t*)(buf + offset), (size_t)(size - offset)),
					 fuseFileInfoToJSObject(env, fileInfo)}
				);
			},
			[](const Napi::Value& value) {
				return value.As<Napi::Number>().Int32Value();
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::unlink(const std::string& path) {
	const auto methodName = "FuseUserData::unlink";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (unlinkCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			unlinkCallback.value(),
			[&path](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path)});
			},
			[](const Napi::Value& value) {
				return value.As<Napi::Number>().Int32Value();
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::chmod(const std::string& path, mode_t mode) {
	const auto methodName = "FuseUserData::chmod";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (chmodCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			chmodCallback.value(),
			[&path, mode](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path), Napi::Number::New(env, mode)});
			},
			[](const Napi::Value& value) {
				return value.As<Napi::Number>().Int32Value();
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::chown(const std::string& path, uid_t user, gid_t group) {
	const auto methodName = "FuseUserData::chown";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (chownCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			chownCallback.value(),
			[&path, user, group](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path), Napi::Number::New(env, user), Napi::Number::New(env, group)});
			},
			[](const Napi::Value& value) {
				return value.As<Napi::Number>().Int32Value();
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}

int FuseUserData::release(const std::string& path, struct fuse_file_info* fileInfo) {
	const auto methodName = "FuseUserData::release";
	trace() << methodName << " begin, path = " << path;
	int result = -ENOENT;
	if (releaseCallback.has_value()) {
		trace() << methodName << " invoking callback";
		result = await<int>(
			releaseCallback.value(),
			[&path, fileInfo](const Napi::Env& env, Napi::Function f) {
				return f({Napi::String::From(env, path), fuseFileInfoToJSObject(env, fileInfo)});
			},
			[](const Napi::Value& value) {
				return value.As<Napi::Number>().Int32Value();
			}
		);
	} else {
		trace() << methodName << " no callback provided";
	}
	trace() << methodName << " end, result = " << result;
	return result;
}