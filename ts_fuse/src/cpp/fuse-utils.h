#pragma once

#include <optional>

#include "common.h"

class FuseUserData {
   private:
	std::optional<Napi::ThreadSafeFunction> initCallback;
	std::optional<Napi::ThreadSafeFunction> destroyCallback;
	std::optional<Napi::ThreadSafeFunction> getattrCallback;
	std::optional<Napi::ThreadSafeFunction> readdirCallback;
	std::optional<Napi::ThreadSafeFunction> openCallback;
	std::optional<Napi::ThreadSafeFunction> readCallback;
	std::optional<Napi::ThreadSafeFunction> writeCallback;
	std::optional<Napi::ThreadSafeFunction> createCallback;
	std::optional<Napi::ThreadSafeFunction> unlinkCallback;
	std::optional<Napi::ThreadSafeFunction> chmodCallback;
	std::optional<Napi::ThreadSafeFunction> chownCallback;
	std::optional<Napi::ThreadSafeFunction> releaseCallback;

	bool destroyed;

   public:
	FuseUserData(const Napi::Env& env, const Napi::Object& callbacks);
	~FuseUserData();

	void init(fuse_conn_info* connectionInfo);
	void destroy();
	int getattr(const std::string& path, struct stat* stat);
	int readdir(const std::string& path, void* buf, fuse_fill_dir_t filler);
	int open(const std::string& path, struct fuse_file_info* fileInfo);
	int read(const std::string& path, char* buf, size_t size, off_t offset, struct fuse_file_info* fileInfo);
	int write(const std::string& path, const char* buf, size_t size, off_t offset, struct fuse_file_info* fileInfo);
	int create(const std::string& path, mode_t mode, struct fuse_file_info* fileInfo);
	int unlink(const std::string& path);
	int chmod(const std::string& path, mode_t mode);
	int chown(const std::string& path, uid_t user, gid_t group);
	int release(const std::string& path, struct fuse_file_info* fileInfo);
};
