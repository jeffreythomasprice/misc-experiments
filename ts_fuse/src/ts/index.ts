import { dirname } from "path";

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

interface InMemoryFile {
	stat: Fuse.Stat;
	data?: Buffer;
}

class InMemoryOpenFile {
	private _position: number;

	constructor(public buffer: Buffer) {
		this._position = 0;
	}

	get position() {
		return this._position;
	}

	read(destination: Buffer): number {
		const remaining = this.buffer.length - this.position;
		const result = Math.min(remaining, destination.length);
		this.buffer.copy(destination, 0, this.position, result);
		this._position += result;
		return result;
	}

	write(source: Buffer): number {
		const end = this.position + source.length;
		const newLength = Math.max(end, this.buffer.length);
		const newBuffer = Buffer.alloc(this.buffer.length);
		this.buffer.copy(newBuffer);
		source.copy(newBuffer, 0, this.position);
		this.buffer = source;
		this._position += source.length;
		return source.length;
	}
}

class InMemoryFileSystem implements FileSystem<InMemoryOpenFile> {
	private readonly nodes = new Map<string, InMemoryFile>();

	init(connectionInfo: Fuse.ConnectionInfo): MaybePromise<void> {
		this.nodes.clear();
		this.nodes.set(
			"/",
			{
				stat: directoryStat({
					mode: FileType.IFDIR | 0o777,
					statusChangeTime: new Date(),
					lastAccessTime: new Date(),
					modificationTime: new Date(),
				})
			}
		);
	}

	destroy(): MaybePromise<void> { }

	getattr(path: string): MaybePromise<Fuse.Stat | undefined | null> {
		return this.nodes.get(path)?.stat;
	}

	readdir(path: string): MaybePromise<Fuse.ReaddirResult[] | undefined | null> {
		const node = this.nodes.get(path);
		if (!node) {
			return undefined;
		}
		if (node.stat.st_mode & FileType.IFDIR) {
			const results: Fuse.ReaddirResult[] = [
				{ path: "." },
				{ path: ".." },
			];
			for (const [childPath, childStat] of this.nodes.entries()) {
				if (childPath !== path && dirname(childPath) === path) {
					results.push({ path: childPath, stat: childStat.stat });
				}
			}
			return results;
		}
	}

	create(path: string, mode: number, fileInfo: Fuse.FileInfo): MaybePromise<InMemoryOpenFile | null | undefined> {
		const data = Buffer.alloc(0);
		this.nodes.set(path, {
			stat: fileStat({
				mode,
				size: 0,
				statusChangeTime: new Date(),
				lastAccessTime: new Date(),
				modificationTime: new Date(),
			}),
			data,
		});
		return new InMemoryOpenFile(data);
	}

	open(path: string, fileInfo: Fuse.FileInfo): MaybePromise<InMemoryOpenFile | undefined | null> {
		const node = this.nodes.get(path);
		if (!node) {
			throw new ErrnoException(Errno.ENOENT);
		}
		if (!node.data) {
			throw new ErrnoException(Errno.EIO);
		}

		// TODO how to prevent reading files we shouldn't have access to?
		if ((fileInfo.flags & FileFlag.ACCMODE) != FileFlag.RDONLY) {
			throw new ErrnoException(Errno.EACCES);
		}

		return new InMemoryOpenFile(node.data);
	}

	read(path: string, buffer: Buffer, fileHandle: InMemoryOpenFile, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null> {
		return fileHandle.read(buffer);
	}

	write(path: string, buffer: Buffer, fileHandle: InMemoryOpenFile, fileInfo: Fuse.FileInfo): MaybePromise<number | null | undefined> {
		return fileHandle.write(buffer);
	}

	unlink(path: string): MaybePromise<void> {
		this.nodes.delete(path);
	}

	chmod(path: string, mode: number): MaybePromise<void> {
		const node = this.nodes.get(path);
		if (!node) {
			throw new ErrnoException(Errno.ENOENT);
		}
		node.stat.st_mode = mode;
	}

	chown(path: string, user: number, group: number): MaybePromise<void> {
		const node = this.nodes.get(path);
		if (!node) {
			throw new ErrnoException(Errno.ENOENT);
		}
		node.stat.st_uid = user;
		node.stat.st_gid = group;
	}

	release(path: string, fileHandle: InMemoryOpenFile, fileInfo: Fuse.FileInfo): MaybePromise<void> {
		const node = this.nodes.get(path);
		if (!node) {
			throw new ErrnoException(Errno.ENOENT);
		}
		node.data = fileHandle.buffer;
	}
}

(async () => {
	getLogger().level = LogLevel.DEBUG;

	const addonLogger = getLogger("native");
	addonLogger.level = LogLevel.TRACE;
	addonInit({
		log: (timestamp, level, message) => {
			addonLogger.log(new Date(timestamp), level, message);
		},
	});

	const mount = await mountAndRun(
		"experiment",
		"/home/jeff/mount_points/test",
		new InMemoryFileSystem()
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
