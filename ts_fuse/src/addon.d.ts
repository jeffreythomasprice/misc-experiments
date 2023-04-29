module "*/addon" {
	export enum LogLevel {
		FATAL = 1,
		ERROR = 2,
		WARN = 3,
		INFO = 4,
		DEBUG = 5,
		TRACE = 6,
	}

	export enum Errno {
		EPERM = 1,
		ENOENT = 2,
		ESRCH = 3,
		EINTR = 4,
		EIO = 5,
		ENXIO = 6,
		E2BIG = 7,
		ENOEXEC = 8,
		EBADF = 9,
		ECHILD = 10,
		EAGAIN = 11,
		ENOMEM = 12,
		EACCES = 13,
		EFAULT = 14,
		ENOTBLK = 15,
		EBUSY = 16,
		EEXIST = 17,
		EXDEV = 18,
		ENODEV = 19,
		ENOTDIR = 20,
		EISDIR = 21,
		EINVAL = 22,
		ENFILE = 23,
		EMFILE = 24,
		ENOTTY = 25,
		ETXTBSY = 26,
		EFBIG = 27,
		ENOSPC = 28,
		ESPIPE = 29,
		EROFS = 30,
		EMLINK = 31,
		EPIPE = 32,
		EDOM = 33,
		ERANGE = 34,
	}

	export enum FileType {
		IFDIR = 0040000,
		IFCHR = 0020000,
		IFBLK = 0060000,
		IFREG = 0100000,
		IFIFO = 0010000,
		IFLNK = 0120000,
		IFSOCK = 0140000,
	}

	export type LogCallback = (timestamp: number, level: LogLevel, message: string) => void;

	export interface InitOptions {
		// a callback to invoke for each log message
		log?: LogCallback;
		// how many log events to hold in memory before trying to emit them all at once to the callback
		// defaults to emitting immediately, i.e. 0
		maxBufferSize?: number;
	}

	export namespace Fuse {
		export interface ConnectionInfo {
			proto_major: number;
			proto_minor: number;
			async_read: number;
			max_write: number;
			max_readahead: number;
			capable: number;
			want: number;
			max_background: number;
			congestion_threshold: number;
		}

		export interface Timespec {
			tv_sec: number;
			tv_nsec: number;
		}

		export interface Stat {
			st_dev: number;
			st_ino: number;
			st_nlink: number;
			st_mode: number;
			st_uid: number;
			st_gid: number;
			st_rdev: number;
			st_size: number;
			st_blksize: number;
			st_blocks: number;
			st_atim: TimeSpec;
			st_mtim: TimeSpec;
			st_ctim: TimeSpec;
		}
	}

	export type MaybePromise<T> = T | Promise<T>;

	export interface MountAndRunCallbacks {
		init?: (connectionInfo: FuseConnectionInfo) => MaybePromise<void>;
		destroy?: () => MaybePromise<void>;
		getattr?: (path: string) => MaybePromise<Errno | Fuse.Stat>;
	}

	export interface FuseMount {
		// returns the result of the fuse_loop
		close(): Promise<number>;
	}

	export function init(options?: InitOptions): void;
	// emits log events before closing
	export function close(): Promise<void>;

	export function mountAndRun(args: string[], callbacks: MountAndRunCallbacks): Promise<FuseMount>;
}
