import {
	Errno,
	FileFlag,
	FileType,
	Fuse,
	LogLevel,
	MaybePromise,
	close as addonClose,
	init as addonInit
} from "../build/Release/addon";

import { ErrnoException } from "./errors";
import { FileSystem, mountAndRun } from "./filesystem";
import { getLogger } from "./logging";
import { directoryStat, fileStat } from "./stat";

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
				return directoryStat({
					mode: FileType.IFDIR | 0o755,
					lastAccessTime: this.fileTime,
					modificationTime: this.fileTime,
					statusChangeTime: this.fileTime,
				});
			case "/test":
				return fileStat({
					mode: FileType.IFREG | 0o444,
					lastAccessTime: this.fileTime,
					modificationTime: this.fileTime,
					statusChangeTime: this.fileTime,
					size: this.contents.length,
				});
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

	write(path: string, buffer: Buffer, fileHandle: ReadOnlyInMemoryFileHandle, fileInfo: Fuse.FileInfo): MaybePromise<number | null | undefined> {
		throw new Error("TODO JEFF implement me");
	}

	create(path: string, mode: number, fileInfo: Fuse.FileInfo): MaybePromise<ReadOnlyInMemoryFileHandle | null | undefined> {
		throw new Error("TODO JEFF implement me");
	}

	unlink(path: string): MaybePromise<void> {
		throw new Error("TODO JEFF implement me");
	}

	chmod(path: string, mode: number): MaybePromise<void> {
		throw new Error("TODO JEFF implement me");
	}

	chown(path: string, user: number, group: number): MaybePromise<void> {
		throw new Error("TODO JEFF implement me");
	}

	release(path: string, fileInfo: Fuse.FileInfo): MaybePromise<void> {
		throw new Error("TODO JEFF implement me");
	}
}

(async () => {
	getLogger().level = LogLevel.DEBUG;

	const addonLogger = getLogger("native");
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
