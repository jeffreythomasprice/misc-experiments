import addon, { Errno, FileType, Fuse, LogLevel, MaybePromise } from "../build/Release/addon";

import { ErrnoException, wrapErrnoCallback } from "./errors";
import { getLogger } from "./logging";

const logger = getLogger();
logger.level = LogLevel.TRACE;

// TODO JEFF move me
interface FileSystem {
	init(connectionInfo: Fuse.ConnectionInfo): MaybePromise<void>;
	destroy(): MaybePromise<void>;
	getattr(path: string): MaybePromise<Fuse.Stat | undefined | null>;
	readdir(path: string): MaybePromise<Fuse.ReaddirResult[] | undefined | null>;
	open(path: string, fileInfo: Fuse.FileInfo): MaybePromise<Fuse.OpenResult | undefined | null>;
	read(path: string, buffer: Buffer, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null>;
}

// TODO JEFF move me
function mountAndRun(name: string, path: string, fs: FileSystem) {
	return addon.mountAndRun(
		[
			name,
			path,
			// foreground mode
			"-f",
		],
		{
			init: async (connectionInfo) => {
				try {
					logger.debug(`init ${JSON.stringify(connectionInfo)}`);
					await fs.init(connectionInfo);
				} catch (e) {
					logger.error("error", e);
				}
			},
			destroy: async () => {
				try {
					logger.debug("destroy");
					await fs.destroy();
				} catch (e) {
					logger.error("error", e);
				}
			},
			getattr: (path) => wrapErrnoCallback(
				`getattr(${path})`,
				async () => {
					const result = await fs.getattr(path);
					if (result) {
						return result;
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
			readdir: (path) => wrapErrnoCallback(
				`readdir(${path})`,
				async () => {
					const result = await fs.readdir(path);
					if (result) {
						return result;
					}
					throw new ErrnoException(Errno.ENOENT);
				},
			),
			// TODO JEFF do some file handle wrapping?
			open: (path, fileInfo) => wrapErrnoCallback(
				`open(${path})`,
				async () => {
					const result = await fs.open(path, fileInfo);
					if (result) {
						return result;
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
			read: (path, buffer, fileInfo) => wrapErrnoCallback(
				`read(${path})`,
				async () => {
					const result = await fs.read(path, buffer, fileInfo);
					if (typeof result === "number") {
						return result;
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
		}
	);
}

class HelloWorldFileSystem implements FileSystem {
	private readonly contents: Buffer;

	constructor(contents: string | Buffer) {
		if (typeof contents === "string") {
			this.contents = Buffer.from(contents, "utf-8");
		} else {
			this.contents = contents;
		}
	}

	init(connectionInfo: Fuse.ConnectionInfo): addon.MaybePromise<void> { }

	destroy(): addon.MaybePromise<void> { }

	getattr(path: string): addon.MaybePromise<Fuse.Stat | undefined | null> {
		switch (path) {
			case "/":
				return {
					st_mode: FileType.IFDIR | 0o755,
					st_nlink: 2,
					// unused
					st_dev: 0,
					st_ino: 0,
					st_uid: 0,
					st_gid: 0,
					st_rdev: 0,
					st_size: 0,
					st_blksize: 0,
					st_blocks: 0,
					st_atim: {
						tv_sec: 0,
						tv_nsec: 0,
					},
					st_mtim: {
						tv_sec: 0,
						tv_nsec: 0,
					},
					st_ctim: {
						tv_sec: 0,
						tv_nsec: 0,
					},
				};
			case "/test":
				return {
					st_mode: FileType.IFREG | 0o444,
					st_nlink: 1,
					st_size: this.contents.length,
					// unused
					st_dev: 0,
					st_ino: 0,
					st_uid: 0,
					st_gid: 0,
					st_rdev: 0,
					st_blksize: 0,
					st_blocks: 0,
					st_atim: {
						tv_sec: 0,
						tv_nsec: 0,
					},
					st_mtim: {
						tv_sec: 0,
						tv_nsec: 0,
					},
					st_ctim: {
						tv_sec: 0,
						tv_nsec: 0,
					},
				};
		}
	}

	readdir(path: string): addon.MaybePromise<Fuse.ReaddirResult[] | undefined | null> {
		if (path === "/") {
			return [
				{ path: "." },
				{ path: ".." },
				{ path: "test" }
			];
		}
	}

	open(path: string, fileInfo: Fuse.FileInfo): addon.MaybePromise<Fuse.OpenResult | undefined | null> {
		if (path === "/test") {
			if ((fileInfo.flags & addon.FileFlag.ACCMODE) != addon.FileFlag.RDONLY) {
				throw new ErrnoException(addon.Errno.EACCES);
			}

			// TODO JEFF use a meaningful file handle value
			return { fh: 42 };
		}
	}

	read(path: string, buffer: Buffer, fileInfo: Fuse.FileInfo): addon.MaybePromise<number | undefined | null> {
		if (path === "/test") {
			// TODO JEFF should be remembering offset on the file handle so we can do partial reads
			return this.contents.copy(buffer);
		}
	}
}

(async () => {
	const addonLogger = getLogger("c++");
	addonLogger.level = LogLevel.DEBUG;
	addon.init({
		log: (timestamp, level, message) => {
			addonLogger.log(new Date(timestamp), level, message);
		},
	});

	const mount = await mountAndRun(
		"experiment",
		"/home/jeff/mount_points/test",
		new HelloWorldFileSystem("Hello, World!\n")
	);
	logger.debug("mounted");

	await new Promise<void>((resolve) => {
		process.once("SIGINT", () => {
			resolve();
		});
	});

	const mountResult = await mount.close();
	logger.debug(`unmounted, result = ${mountResult}`);

	await addon.close();
	process.exit(0);
})()
	.catch((e) => {
		logger.fatal("error", e);
		process.exit(1);
	});
