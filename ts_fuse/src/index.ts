import addon, { Errno, FileType, LogLevel } from "../build/Release/addon";

import { ErrnoException, wrapErrnoCallback } from "./errors";
import { getLogger } from "./logging";

const logger = getLogger();
logger.level = LogLevel.TRACE;

(async () => {
	const addonLogger = getLogger("c++");
	addonLogger.level = LogLevel.DEBUG;
	addon.init({
		log: (timestamp, level, message) => {
			addonLogger.log(new Date(timestamp), level, message);
		},
	});

	const testFileContents = Buffer.from("Hello, World!", "ascii");

	const mount = await addon.mountAndRun(
		[
			"experiment",
			"/home/jeff/mount_points/test",
			// foreground mode
			"-f",
		],
		{
			init: (connectionInfo) => {
				try {
					logger.debug(`init ${JSON.stringify(connectionInfo)}`);
				} catch (e) {
					logger.error("error", e);
				}
			},
			destroy: () => {
				try {
					logger.debug("destroy");
				} catch (e) {
					logger.error("error", e);
				}
			},
			getattr: (path) => wrapErrnoCallback(
				`getattr(${path})`,
				() => {
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
								st_size: testFileContents.length,
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
						default:
							throw new ErrnoException(Errno.ENOENT);
					}
				}
			),
			readdir: (path) => wrapErrnoCallback(
				`readdir(${path})`,
				() => {
					if (path === "/") {
						return [
							{ path: "." },
							{ path: ".." },
							{ path: "test" }
						];
					}
					throw new ErrnoException(addon.Errno.ENOENT);
				}
			),
			open: (path, fileInfo) => wrapErrnoCallback(
				`open(${path})`,
				() => {
					logger.debug("open", path, JSON.stringify(fileInfo));
					if (path === "/test") {
						if ((fileInfo.flags & addon.FileFlag.ACCMODE) != addon.FileFlag.RDONLY) {
							throw new ErrnoException(addon.Errno.EACCES);
						}

						// TODO JEFF use a meaningful file handle value
						return { fh: 42 };
					}
					throw new ErrnoException(addon.Errno.ENOENT);
				}
			),
			read: (path, buffer, fileInfo) => wrapErrnoCallback(
				`read(${path})`,
				() => {
					logger.debug("read", path, JSON.stringify(fileInfo));
					if (path === "/test") {
						// TODO JEFF should be remembering offset on the file handle so we can do partial reads
						return testFileContents.copy(buffer);
					}
					throw new ErrnoException(addon.Errno.ENOENT);
				}
			),
		}
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
