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
					env,
					log.As<Napi::Function>(),
					"unbuffer logs",
					// max queue size, 0 = unlimited
					0,
					// initial thread count
					1
				);
			}
		}

		if (options.Has("maxBufferSize")) {
			auto maxBufferSize = options.Get("maxBufferSize");
			if (maxBufferSize.IsNumber()) {
				maxBufferSizeBeforeFlush = maxBufferSize.As<Napi::Number>().Int64Value();
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
	return execInNewThread(
		env,
		[]() {},
		[](const Napi::Env& env) {
			deinitLogging();

			isInit = false;

			trace() << "close done";

			return env.Undefined();
		}
	);
}

void* fuseInitImpl(fuse_conn_info* connectionInfo) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	data->init(connectionInfo);
	return data;
}

void fuseDestroyImpl(void*) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	data->destroy();
}

int fuseGetattrImpl(const char* path, struct stat* stat) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	return data->getattr(path, stat);
}

int fuseReaddirImpl(const char* path, void* buf, fuse_fill_dir_t filler, off_t, struct fuse_file_info*) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	return data->readdir(path, buf, filler);
}

int fuseOpenImpl(const char* path, struct fuse_file_info* fileInfo) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	return data->open(path, fileInfo);
}

int fuseReadImpl(const char* path, char* buf, size_t size, off_t offset, struct fuse_file_info* fileInfo) {
	auto context = fuse_get_context();
	auto data = (FuseUserData*)context->private_data;
	return data->read(path, buf, size, offset, fileInfo);
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

	return execInNewThread(
		env,
		[]() {},
		[fuseArgs, fuseUserData](const Napi::Env& env) {
			char* mountPoint;
			int multithreaded;
			int foreground;
			fuse_parse_cmdline(fuseArgs, &mountPoint, &multithreaded, &foreground);
			trace() << "mountAndRun mountPoint=" << mountPoint << ", multithreaded=" << multithreaded << ", foreground=" << foreground;

			auto fuseChannel = fuse_mount(mountPoint, fuseArgs);

			auto fuseOperations = new fuse_operations;
			memset(fuseOperations, 0, sizeof(fuse_operations));
			fuseOperations->init = fuseInitImpl;
			fuseOperations->destroy = fuseDestroyImpl;
			fuseOperations->getattr = fuseGetattrImpl;
			fuseOperations->readdir = fuseReaddirImpl;
			fuseOperations->open = fuseOpenImpl;
			fuseOperations->read = fuseReadImpl;
			// TODO more operations

			auto fuseInstance = fuse_new(fuseChannel, fuseArgs, fuseOperations, sizeof(fuse_operations), fuseUserData);

			auto fuseLoopThreadResult = new int;
			auto fuseLoopThread = new std::thread([mountPoint, fuseInstance, fuseLoopThreadResult]() {
				trace() << "mount point " << mountPoint << " fuse_loop begin";
				*fuseLoopThreadResult = fuse_loop(fuseInstance);
				trace() << "mount point " << mountPoint << " fuse_loop done, result = " << *fuseLoopThreadResult;
			});

			auto result = Napi::Object::New(env);
			result.Set(
				"close",
				Napi::Function::New(
					env,
					[fuseArgs, fuseUserData, fuseOperations, mountPoint, fuseChannel, fuseInstance, fuseLoopThread, fuseLoopThreadResult](
						const Napi::CallbackInfo& info
					) {
						auto env = info.Env();
						return execInNewThread(
							env,
							[mountPoint, fuseChannel, fuseInstance, fuseOperations, fuseArgs, fuseUserData, fuseLoopThread]() {
								trace() << "mount point " << mountPoint << " unmount begin";

								fuse_unmount(mountPoint, fuseChannel);
								trace() << "mount point " << mountPoint << " unmount fuse_unmount complete";

								fuse_exit(fuseInstance);
								trace() << "mount point " << mountPoint << " unmount fuse_exit complete";

								delete fuseOperations;

								for (auto i = 0; i < fuseArgs->argc; i++) {
									delete fuseArgs->argv[i];
								}
								delete fuseArgs->argv;

								delete fuseUserData;

								fuseLoopThread->join();
								delete fuseLoopThread;
							},
							[mountPoint, fuseLoopThreadResult](const Napi::Env& env) {
								auto result = *fuseLoopThreadResult;
								delete fuseLoopThreadResult;
								trace() << "mount point " << mountPoint << " unmount fuse_loop complete, result = " << result;

								debug() << "unmounted " << mountPoint;

								trace() << "mount point " << mountPoint << " unmount end";
								return Napi::Number::From(env, result);
							}
						);
					}
				)
			);
			result.Freeze();

			debug() << "mounted " << mountPoint;
			trace() << "mountAndRun done";
			return result;
		}
	);
}

Napi::Object init(Napi::Env env, Napi::Object exports) {
	initThreadUtils(env);

	auto logLevels = Napi::Object::New(env);
	logLevels.Set("FATAL", Napi::Number::New(env, (int)LogLevel::FATAL));
	logLevels.Set("ERROR", Napi::Number::New(env, (int)LogLevel::ERROR));
	logLevels.Set("WARN", Napi::Number::New(env, (int)LogLevel::WARN));
	logLevels.Set("INFO", Napi::Number::New(env, (int)LogLevel::INFO));
	logLevels.Set("DEBUG", Napi::Number::New(env, (int)LogLevel::DEBUG));
	logLevels.Set("TRACE", Napi::Number::New(env, (int)LogLevel::TRACE));
	logLevels.Freeze();
	exports.Set(Napi::String::New(env, "LogLevel"), logLevels);

	auto errnos = Napi::Object::New(env);
	errnos.Set("EPERM", Napi::Number::New(env, EPERM));
	errnos.Set("ENOENT", Napi::Number::New(env, ENOENT));
	errnos.Set("ESRCH", Napi::Number::New(env, ESRCH));
	errnos.Set("EINTR", Napi::Number::New(env, EINTR));
	errnos.Set("EIO", Napi::Number::New(env, EIO));
	errnos.Set("ENXIO", Napi::Number::New(env, ENXIO));
	errnos.Set("E2BIG", Napi::Number::New(env, E2BIG));
	errnos.Set("ENOEXEC", Napi::Number::New(env, ENOEXEC));
	errnos.Set("EBADF", Napi::Number::New(env, EBADF));
	errnos.Set("ECHILD", Napi::Number::New(env, ECHILD));
	errnos.Set("EAGAIN", Napi::Number::New(env, EAGAIN));
	errnos.Set("ENOMEM", Napi::Number::New(env, ENOMEM));
	errnos.Set("EACCES", Napi::Number::New(env, EACCES));
	errnos.Set("EFAULT", Napi::Number::New(env, EFAULT));
	errnos.Set("ENOTBLK", Napi::Number::New(env, ENOTBLK));
	errnos.Set("EBUSY", Napi::Number::New(env, EBUSY));
	errnos.Set("EEXIST", Napi::Number::New(env, EEXIST));
	errnos.Set("EXDEV", Napi::Number::New(env, EXDEV));
	errnos.Set("ENODEV", Napi::Number::New(env, ENODEV));
	errnos.Set("ENOTDIR", Napi::Number::New(env, ENOTDIR));
	errnos.Set("EISDIR", Napi::Number::New(env, EISDIR));
	errnos.Set("EINVAL", Napi::Number::New(env, EINVAL));
	errnos.Set("ENFILE", Napi::Number::New(env, ENFILE));
	errnos.Set("EMFILE", Napi::Number::New(env, EMFILE));
	errnos.Set("ENOTTY", Napi::Number::New(env, ENOTTY));
	errnos.Set("ETXTBSY", Napi::Number::New(env, ETXTBSY));
	errnos.Set("EFBIG", Napi::Number::New(env, EFBIG));
	errnos.Set("ENOSPC", Napi::Number::New(env, ENOSPC));
	errnos.Set("ESPIPE", Napi::Number::New(env, ESPIPE));
	errnos.Set("EROFS", Napi::Number::New(env, EROFS));
	errnos.Set("EMLINK", Napi::Number::New(env, EMLINK));
	errnos.Set("EPIPE", Napi::Number::New(env, EPIPE));
	errnos.Set("EDOM", Napi::Number::New(env, EDOM));
	errnos.Set("ERANGE", Napi::Number::New(env, ERANGE));
	errnos.Freeze();
	exports.Set(Napi::String::New(env, "Errno"), errnos);

	auto fileTypes = Napi::Object::New(env);
	fileTypes.Set("IFDIR", Napi::Number::New(env, S_IFDIR));
	fileTypes.Set("IFCHR", Napi::Number::New(env, S_IFCHR));
	fileTypes.Set("IFBLK", Napi::Number::New(env, S_IFBLK));
	fileTypes.Set("IFREG", Napi::Number::New(env, S_IFREG));
	fileTypes.Set("IFIFO", Napi::Number::New(env, S_IFIFO));
	fileTypes.Set("IFLNK", Napi::Number::New(env, S_IFLNK));
	fileTypes.Set("IFSOCK", Napi::Number::New(env, S_IFSOCK));
	fileTypes.Freeze();
	exports.Set(Napi::String::New(env, "FileType"), fileTypes);

	exports.Set(Napi::String::New(env, "init"), Napi::Function::New(env, exportedInit));
	exports.Set(Napi::String::New(env, "close"), Napi::Function::New(env, exportedClose));
	exports.Set(Napi::String::New(env, "mountAndRun"), Napi::Function::New(env, exportedMountAndRun));
	return exports;
}

NODE_API_MODULE(addon, init)