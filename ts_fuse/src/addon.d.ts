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
			readonly proto_major: number;
			readonly proto_minor: number;
			readonly async_read: number;
			readonly max_write: number;
			readonly max_readahead: number;
			readonly capable: number;
			readonly want: number;
			readonly max_background: number;
			readonly congestion_threshold: number;
		}

		export interface Timespec {
			readonly tv_sec: number;
			readonly tv_nsec: number;
		}

		export interface Stat {
			readonly st_dev: number;
			readonly st_ino: number;
			readonly st_nlink: number;
			readonly st_mode: number;
			readonly st_uid: number;
			readonly st_gid: number;
			readonly st_rdev: number;
			readonly st_size: number;
			readonly st_blksize: number;
			readonly st_blocks: number;
			readonly st_atim: TimeSpec;
			readonly st_mtim: TimeSpec;
			readonly st_ctim: TimeSpec;
		}

		export interface ReaddirResult {
			readonly path: string;
			readonly stat?: Stat;
		}

		export interface FileInfo {
			readonly flags: number;
			readonly writepage: number;
			readonly direct_io: boolean;
			readonly keep_cache: boolean;
			readonly flush: boolean;
			readonly nonseekable: boolean;
			readonly flock_release: boolean;
			readonly fh: number;
			readonly lock_owner: number;
		}

		export interface OpenResult {
			readonly fh: number;
		}
	}

	export type MaybePromise<T> = T | Promise<T>;

	export interface MountAndRunCallbacks {
		init?: (connectionInfo: FuseConnectionInfo) => MaybePromise<void>;
		destroy?: () => MaybePromise<void>;
		getattr?: (path: string) => MaybePromise<Errno | Fuse.Stat>;
		readdir?: (path: string) => MaybePromise<Errno | ReaddirResult[]>;
		open?: (path: string, fileInfo: FileInfo) => MaybePromise<Errno | OpenResult>;
		read?: (path: string, buffer: Buffer, fileInfo: FileInfo) => MaybePromise<number>;
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
