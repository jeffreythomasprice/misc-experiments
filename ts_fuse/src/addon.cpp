#include <assert.h>

#include <sstream>
#include <thread>

#include "fuse-utils.h"
#include "logging.h"
#include "thread-utils.h"

std::mutex initMutex;
bool isInit = false;

Napi::Value exportedInit(const Napi::CallbackInfo& info) {
	trace() << "init begin";

	auto env = info.Env();

	{
		std::unique_lock lock(initMutex);
		if (isInit) {
			throw Napi::Error::New(env, "init already in progress");
		}
		isInit = true;
	}

	std::optional<Napi::ThreadSafeFunction> emitLogCallback = std::nullopt;
	auto maxBufferSizeBeforeFlush = 0;
	if (info.Length() >= 1 && info[0].IsObject()) {
		auto options = info[0].As<Napi::Object>();

		if (options.Has("log")) {
			auto log = options.Get("log");
			if (log.IsFunction()) {
				emitLogCallback = Napi::ThreadSafeFunction::New(
					env, log.As<Napi::Function>(), "unbuffer logs",
					// max queue size, 0 = unlimited
					0,
					// initial thread count
					1);
			}
		}

		if (options.Has("maxBufferSize")) {
			auto maxBufferSize = options.Get("maxBufferSize");
			if (maxBufferSize.IsNumber()) {
				maxBufferSizeBeforeFlush =
					maxBufferSize.As<Napi::Number>().Int64Value();
			}
		}
	}
	initLogging(maxBufferSizeBeforeFlush, emitLogCallback);

	trace() << "init done";

	return env.Undefined();
}

Napi::Value exportedClose(const Napi::CallbackInfo& info) {
	trace() << "close begin";

	auto env = info.Env();

	std::unique_lock lock(initMutex);

	unbufferLogs();

	// have to finish after letting the node event loop finish
	// this lets all those log messages finish unbuffering and emitting back to
	// node before we actually clean up
	return execInNewThread(env, [](const Napi::Env& env) {
		deinitLogging();

		isInit = false;

		trace() << "close done";

		return env.Undefined();
	});
}

void* fuseInitImpl(fuse_conn_info* connectionInfo) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	data->init(connectionInfo);
	return nullptr;
}

void fuseDestroyImpl(void*) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	data->destroy();
}

int fuseGetattrImpl(const char* path, struct stat* stat) {
	trace() << "fuseGetattrImpl begin, path = " << path;
	// TODO implement getattr
	auto result = -ENOENT;
	trace() << "fuseGetattrImpl end, result " << result;
	return result;
}

int fuseReaddirImpl(const char* path, void*, fuse_fill_dir_t, off_t,
					struct fuse_file_info*) {
	trace() << "fuseReaddirImpl begin, path = " << path;
	// TODO implement readdir
	auto result = -ENOENT;
	trace() << "fuseReaddirImpl end, result " << result;
	return result;
}

Napi::Value exportedMountAndRun(const Napi::CallbackInfo& info) {
	trace() << "mountAndRun begin";

	auto env = info.Env();

	auto jsArgs = info[0].As<Napi::Array>();
	auto fuseArgs = new fuse_args;
	fuseArgs->allocated = 0;
	fuseArgs->argc = jsArgs.Length();
	fuseArgs->argv = new char*[fuseArgs->argc];
	for (size_t i = 0; i < jsArgs.Length(); i++) {
		std::string arg = jsArgs.Get(i).As<Napi::String>();
		trace() << "mountAndRun arg[" << i << "] = " << arg;
		fuseArgs->argv[i] = new char[arg.size() + 1];
		strcpy(fuseArgs->argv[i], arg.c_str());
	}

	auto callbacks = info[1].As<Napi::Object>();
	auto fuseUserData = new FuseUserData(env, callbacks);

	return execInNewThread(env, [fuseArgs, fuseUserData](const Napi::Env& env) {
		char* mountPoint;
		int multithreaded;
		int foreground;
		fuse_parse_cmdline(fuseArgs, &mountPoint, &multithreaded, &foreground);
		trace() << "mountAndRun mountPoint=" << mountPoint
				<< ", multithreaded=" << multithreaded
				<< ", foreground=" << foreground;

		auto fuseChannel = fuse_mount(mountPoint, fuseArgs);

		auto fuseOperations = new fuse_operations;
		memset(fuseOperations, 0, sizeof(fuse_operations));
		fuseOperations->init = fuseInitImpl;
		fuseOperations->destroy = fuseDestroyImpl;
		fuseOperations->getattr = fuseGetattrImpl;
		fuseOperations->readdir = fuseReaddirImpl;
		// TODO more operations

		auto fuseInstance = fuse_new(fuseChannel, fuseArgs, fuseOperations,
									 sizeof(fuse_operations), fuseUserData);

		auto fuseLoopThreadResult = new int;
		auto fuseLoopThread = new std::thread([mountPoint, fuseInstance,
											   fuseLoopThreadResult]() {
			trace() << "mount point " << mountPoint << " fuse_loop begin";
			*fuseLoopThreadResult = fuse_loop(fuseInstance);
			trace() << "mount point " << mountPoint
					<< " fuse_loop done, result = " << *fuseLoopThreadResult;
		});

		auto result = Napi::Object::New(env);
		result.Set(
			"close",
			Napi::Function::New(
				env, [fuseArgs, fuseUserData, fuseOperations, mountPoint,
					  fuseChannel, fuseInstance, fuseLoopThread,
					  fuseLoopThreadResult](const Napi::CallbackInfo& info) {
					trace() << "mount point " << mountPoint << " unmount begin";

					auto env = info.Env();

					fuse_unmount(mountPoint, fuseChannel);
					trace() << "mount point " << mountPoint
							<< " unmount fuse_unmount complete";

					fuse_exit(fuseInstance);
					trace() << "mount point " << mountPoint
							<< " unmount fuse_exit complete";

					delete fuseOperations;

					for (auto i = 0; i < fuseArgs->argc; i++) {
						delete fuseArgs->argv[i];
					}
					delete fuseArgs->argv;

					delete fuseUserData;

					fuseLoopThread->join();
					delete fuseLoopThread;
					auto result = *fuseLoopThreadResult;
					delete fuseLoopThreadResult;
					trace()
						<< "mount point " << mountPoint
						<< " unmount fuse_loop complete, result = " << result;

					debug() << "unmounted " << mountPoint;

					return execInNewThread(
						env, [mountPoint, result](const Napi::Env& env) {
							trace() << "mount point " << mountPoint
									<< " unmount end";
							return Napi::Number::From(env, result);
						});
				}));
		result.Freeze();

		debug() << "mounted " << mountPoint;
		trace() << "mountAndRun done";
		return result;
	});
}

Napi::Object init(Napi::Env env, Napi::Object exports) {
	auto logLevels = Napi::Object::New(env);
	logLevels.Set("FATAL", Napi::Number::New(env, (int)LogLevel::FATAL));
	logLevels.Set("ERROR", Napi::Number::New(env, (int)LogLevel::ERROR));
	logLevels.Set("WARN", Napi::Number::New(env, (int)LogLevel::WARN));
	logLevels.Set("INFO", Napi::Number::New(env, (int)LogLevel::INFO));
	logLevels.Set("DEBUG", Napi::Number::New(env, (int)LogLevel::DEBUG));
	logLevels.Set("TRACE", Napi::Number::New(env, (int)LogLevel::TRACE));
	logLevels.Freeze();
	exports.Set(Napi::String::New(env, "LogLevel"), logLevels);

	exports.Set(Napi::String::New(env, "init"),
				Napi::Function::New(env, exportedInit));
	exports.Set(Napi::String::New(env, "close"),
				Napi::Function::New(env, exportedClose));
	exports.Set(Napi::String::New(env, "mountAndRun"),
				Napi::Function::New(env, exportedMountAndRun));
	return exports;
}

NODE_API_MODULE(addon, init)