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

class ReadOnlyInMemoryFileHandle {
	private _position: number;

	constructor(public readonly buffer: Buffer) {
		this._position = 0;
	}

	get position() {
		return this._position;
	}

	read(destination: Buffer): number {
		const remaining = this.buffer.length - this.position;
		const result = Math.min(remaining, destination.length);
		this.buffer.copy(destination, 0, this.position, result);
		return result;
	}
}

function dateToTimespec(date: Date): Fuse.Timespec {
	const milliseconds = date.valueOf();
	const seconds = Math.floor(milliseconds / 1000);
	return {
		tv_sec: seconds,
		tv_nsec: (milliseconds - seconds * 1000) * 1000000,
	};
}

class HelloWorldFileSystem implements FileSystem<ReadOnlyInMemoryFileHandle> {
	private readonly contents: Buffer;
	private readonly fileTime = new Date();

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
					st_atim: dateToTimespec(this.fileTime),
					st_mtim: dateToTimespec(this.fileTime),
					st_ctim: dateToTimespec(this.fileTime),
					// unused
					st_dev: 0,
					st_ino: 0,
					st_uid: 0,
					st_gid: 0,
					st_rdev: 0,
					st_size: 0,
					st_blksize: 0,
					st_blocks: 0,
				};
			case "/test":
				return {
					st_mode: FileType.IFREG | 0o400,
					st_nlink: 1,
					st_size: this.contents.length,
					st_atim: dateToTimespec(this.fileTime),
					st_mtim: dateToTimespec(this.fileTime),
					st_ctim: dateToTimespec(this.fileTime),
					// unused
					st_dev: 0,
					st_ino: 0,
					st_uid: 0,
					st_gid: 0,
					st_rdev: 0,
					st_blksize: 0,
					st_blocks: 0,
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

	open(path: string, fileInfo: Fuse.FileInfo): MaybePromise<ReadOnlyInMemoryFileHandle | undefined | null> {
		if (path === "/test") {
			// TODO how to prevent reading files we shouldn't have access to?
			if ((fileInfo.flags & FileFlag.ACCMODE) != FileFlag.RDONLY) {
				throw new ErrnoException(Errno.EACCES);
			}
			return new ReadOnlyInMemoryFileHandle(this.contents);
		}
	}

	read(path: string, buffer: Buffer, fileHandle: ReadOnlyInMemoryFileHandle, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null> {
		return fileHandle.read(buffer);
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
