import {
	Errno,
	FileFlag,
	FileType,
	Fuse,
	LogLevel,
	MaybePromise,
	close as addonClose,
	init as addonInit,
} from "../build/Release/addon";

import { ErrnoException } from "./errors";
import { FileSystem, mountAndRun } from "./filesystem";
import { getLogger } from "./logging";

class HelloWorldFileSystem implements FileSystem {
	private readonly contents: Buffer;

	constructor(contents: string | Buffer) {
		if (typeof contents === "string") {
			this.contents = Buffer.from(contents, "utf-8");
		} else {
			this.contents = contents;
		}
	}

	init(connectionInfo: Fuse.ConnectionInfo): MaybePromise<void> { }

	destroy(): MaybePromise<void> { }

	getattr(path: string): MaybePromise<Fuse.Stat | undefined | null> {
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

	readdir(path: string): MaybePromise<Fuse.ReaddirResult[] | undefined | null> {
		if (path === "/") {
			return [
				{ path: "." },
				{ path: ".." },
				{ path: "test" }
			];
		}
	}

	open(path: string, fileInfo: Fuse.FileInfo): MaybePromise<Fuse.OpenResult | undefined | null> {
		if (path === "/test") {
			if ((fileInfo.flags & FileFlag.ACCMODE) != FileFlag.RDONLY) {
				throw new ErrnoException(Errno.EACCES);
			}

			// TODO JEFF use a meaningful file handle value
			return { fh: 42 };
		}
	}

	read(path: string, buffer: Buffer, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null> {
		if (path === "/test") {
			// TODO JEFF should be remembering offset on the file handle so we can do partial reads
			return this.contents.copy(buffer);
		}
	}
}

(async () => {
	getLogger().level = LogLevel.DEBUG;

	const addonLogger = getLogger("c++");
	addonLogger.level = LogLevel.INFO;
	addonInit({
		log: (timestamp, level, message) => {
			addonLogger.log(new Date(timestamp), level, message);
		},
	});

	const mount = await mountAndRun(
		"experiment",
		"/home/jeff/mount_points/test",
		new HelloWorldFileSystem("Hello, World!\n")
	);

	await new Promise<void>((resolve) => {
		process.once("SIGINT", () => {
			resolve();
		});
	});

	await mount.close();

	await addonClose();
	process.exit(0);
})()
	.catch((e) => {
		getLogger().fatal("error", e);
		process.exit(1);
	});
