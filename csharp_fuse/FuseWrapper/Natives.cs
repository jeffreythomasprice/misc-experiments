using System.Runtime.InteropServices;

namespace Experiment.FuseWrapper;

internal static class Natives
{
	public const int EPERM = 1;
	public const int ENOENT = 2;
	public const int ESRCH = 3;
	public const int EINTR = 4;
	public const int EIO = 5;
	public const int ENXIO = 6;
	public const int E2BIG = 7;
	public const int ENOEXEC = 8;
	public const int EBADF = 9;
	public const int ECHILD = 10;
	public const int EAGAIN = 11;
	public const int ENOMEM = 12;
	public const int EACCES = 13;
	public const int EFAULT = 14;
	public const int ENOTBLK = 15;
	public const int EBUSY = 16;
	public const int EEXIST = 17;
	public const int EXDEV = 18;
	public const int ENODEV = 19;
	public const int ENOTDIR = 20;
	public const int EISDIR = 21;
	public const int EINVAL = 22;
	public const int ENFILE = 23;
	public const int EMFILE = 24;
	public const int ENOTTY = 25;
	public const int ETXTBSY = 26;
	public const int EFBIG = 27;
	public const int ENOSPC = 28;
	public const int ESPIPE = 29;
	public const int EROFS = 30;
	public const int EMLINK = 31;
	public const int EPIPE = 32;
	public const int EDOM = 33;
	public const int ERANGE = 34;

	[StructLayout(LayoutKind.Sequential)]
	public struct FuseConnInfo
	{
		public UInt32 proto_major;
		public UInt32 proto_minor;
		public UInt32 async_read;
		public UInt32 max_write;
		public UInt32 max_readahead;
		public UInt32 capable;
		public UInt32 want;
		public UInt32 max_background;
		public UInt32 congestion_threshold;

		[MarshalAs(UnmanagedType.ByValArray, SizeConst = 23)]
		public UInt32[] reserved;
	}

	[StructLayout(LayoutKind.Sequential)]
	public struct FuseFileInfo
	{
		/*
		TODO FuseFileInfo

		int flags;
		unsigned long fh_old;
		int writepage;
		unsigned int direct_io : 1;
		unsigned int keep_cache : 1;
		unsigned int flush : 1;
		unsigned int nonseekable : 1;
		unsigned int flock_release : 1;
		unsigned int padding : 27;
		uint64_t fh;
		uint64_t lock_owner;
		*/
	}

	// int (*getattr) (const char *, struct stat *);
	public delegate int FuseGetAttrFunc(string path, IntPtr stat);

	// typedef int (*fuse_fill_dir_t) (void *buf, const char *name,
	// const struct stat *stbuf, off_t off);
	public delegate int FuseFillDirFunc(IntPtr buf, string name, IntPtr stat, Int64 off);
	// int (*readdir) (const char *, void *, fuse_fill_dir_t, off_t,
	// 		struct fuse_file_info *);
	public delegate int FuseReaddirFunc(string path, IntPtr data, FuseFillDirFunc callback, Int64 off, ref FuseFileInfo info);

	public delegate IntPtr FuseInitFunc(ref FuseConnInfo conn);

	public delegate void FuseDestroyFunc(IntPtr data);

	[StructLayout(LayoutKind.Sequential)]
	public struct FuseOperations
	{
		// int (*getattr) (const char *, struct stat *);
		public FuseGetAttrFunc getattr;

		// int (*readlink) (const char *, char *, size_t);
		public IntPtr readlink;

		// int (*getdir) (const char *, fuse_dirh_t, fuse_dirfil_t);
		public IntPtr getdir;

		// int (*mknod) (const char *, mode_t, dev_t);
		public IntPtr mknod;

		// int (*mkdir) (const char *, mode_t);
		public IntPtr mkdir;

		// int (*unlink) (const char *);
		public IntPtr unlink;

		// int (*rmdir) (const char *);
		public IntPtr rmdir;

		// int (*symlink) (const char *, const char *);
		public IntPtr symlink;

		// int (*rename) (const char *, const char *);
		public IntPtr rename;

		// int (*link) (const char *, const char *);
		public IntPtr link;

		// int (*chmod) (const char *, mode_t);
		public IntPtr chmod;

		// int (*chown) (const char *, uid_t, gid_t);
		public IntPtr chown;

		// int (*truncate) (const char *, off_t);
		public IntPtr truncate;

		// int (*utime) (const char *, struct utimbuf *);
		public IntPtr utime;

		// int (*open) (const char *, struct fuse_file_info *);
		public IntPtr open;

		// int (*read) (const char *, char *, size_t, off_t,
		// 		struct fuse_file_info *);
		public IntPtr read;

		// int (*write) (const char *, const char *, size_t, off_t,
		// 		struct fuse_file_info *);
		public IntPtr write;

		// int (*statfs) (const char *, struct statvfs *);
		public IntPtr statfs;

		// int (*flush) (const char *, struct fuse_file_info *);
		public IntPtr flush;

		// int (*release) (const char *, struct fuse_file_info *);
		public IntPtr release;

		// int (*fsync) (const char *, int, struct fuse_file_info *);
		public IntPtr fsync;

		// int (*setxattr) (const char *, const char *, const char *, size_t, int);
		public IntPtr setxattr;

		// int (*getxattr) (const char *, const char *, char *, size_t);
		public IntPtr getxattr;

		// int (*listxattr) (const char *, char *, size_t);
		public IntPtr listxattr;

		// int (*removexattr) (const char *, const char *);
		public IntPtr removexattr;

		// int (*opendir) (const char *, struct fuse_file_info *);
		public IntPtr opendir;

		// int (*readdir) (const char *, void *, fuse_fill_dir_t, off_t,
		// 		struct fuse_file_info *);
		public FuseReaddirFunc readdir;

		// int (*releasedir) (const char *, struct fuse_file_info *);
		public IntPtr releasedir;

		// int (*fsyncdir) (const char *, int, struct fuse_file_info *);
		public IntPtr fsyncdir;

		// void *(*init) (struct fuse_conn_info *conn);
		public FuseInitFunc? init;

		// void (*destroy) (void *);
		public FuseDestroyFunc? destroy;

		// int (*access) (const char *, int);
		public IntPtr access;

		// int (*create) (const char *, mode_t, struct fuse_file_info *);
		public IntPtr create;

		// int (*ftruncate) (const char *, off_t, struct fuse_file_info *);
		public IntPtr ftruncate;

		// int (*fgetattr) (const char *, struct stat *, struct fuse_file_info *);
		public IntPtr fgetattr;

		// int (*lock) (const char *, struct fuse_file_info *, int cmd,
		// 		struct flock *);
		public IntPtr _lock;

		// int (*utimens) (const char *, const struct timespec tv[2]);
		public IntPtr utimens;

		// int (*bmap) (const char *, size_t blocksize, uint64_t *idx);
		public IntPtr bmap;

		// unsigned int flag_nullpath_ok:1;
		// unsigned int flag_nopath:1;
		// unsigned int flag_utime_omit_ok:1;
		// unsigned int flag_reserved:29;
		public UInt32 flags;

		// int (*ioctl) (const char *, int cmd, void *arg,
		// 		struct fuse_file_info *, unsigned int flags, void *data);
		public IntPtr ioctl;

		// int (*poll) (const char *, struct fuse_file_info *,
		// 		struct fuse_pollhandle *ph, unsigned *reventsp);
		public IntPtr poll;

		// int (*write_buf) (const char *, struct fuse_bufvec *buf, off_t off,
		// 		struct fuse_file_info *);
		public IntPtr write_buf;

		// int (*read_buf) (const char *, struct fuse_bufvec **bufp,
		// 		size_t size, off_t off, struct fuse_file_info *);
		public IntPtr read_buf;

		// int (*flock) (const char *, struct fuse_file_info *, int op);
		public IntPtr flock;

		// int (*fallocate) (const char *, int, off_t, off_t,
		// 		struct fuse_file_info *);
		public IntPtr fallocate;
	}

	[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
	public delegate void LogFunc(string s);

	[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
	public delegate void DataFunc(IntPtr data);

	[DllImport("libfusehelper.so", EntryPoint = "createLogger", CharSet = CharSet.Ansi)]
	public static extern IntPtr CreateLogger(LogFunc trace, LogFunc debug, LogFunc information, LogFunc warning, LogFunc error, LogFunc critical);

	[DllImport("libfusehelper.so", EntryPoint = "freeLogger", CharSet = CharSet.Ansi)]
	public static extern void FreeLogger(IntPtr logger);

	[DllImport("libfusehelper.so", EntryPoint = "createStat")]
	public static extern IntPtr CreateStat();

	[DllImport("libfusehelper.so", EntryPoint = "freeStat")]
	public static extern void FreeStat(IntPtr s);

	[DllImport("libfusehelper.so", EntryPoint = "mountAndRun", CharSet = CharSet.Ansi)]
	public static extern int MountAndRun(IntPtr logger, int argc, string[] argv, ref FuseOperations ops, DataFunc callback);

	[DllImport("libfusehelper.so", EntryPoint = "unmountAndExit")]
	public static extern void UnmountAndExit(IntPtr logger, IntPtr data);
}