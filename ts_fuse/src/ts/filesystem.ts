import addon, { Errno, Fuse, MaybePromise } from "../build/Release/addon";

import { ErrnoException, wrapErrnoCallback } from "./errors";
import { getLogger } from "./logging";

const logger = getLogger("fs");

export interface FileSystem<FileHandle> {
	init(connectionInfo: Fuse.ConnectionInfo): MaybePromise<void>;
	destroy(): MaybePromise<void>;
	getattr(path: string): MaybePromise<Fuse.Stat | undefined | null>;
	readdir(path: string): MaybePromise<Fuse.ReaddirResult[] | undefined | null>;
	create(path: string, mode: number, fileInfo: Fuse.FileInfo): MaybePromise<FileHandle | undefined | null>;
	open(path: string, fileInfo: Fuse.FileInfo): MaybePromise<FileHandle | undefined | null>;
	read(path: string, buffer: Buffer, fileHandle: FileHandle, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null>;
	write(path: string, buffer: Buffer, fileHandle: FileHandle, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null>;
	unlink(path: string): MaybePromise<void>;
	chmod(path: string, mode: number): MaybePromise<void>;
	chown(path: string, user: number, group: number): MaybePromise<void>;
	release(path: string, fileHandle: FileHandle, fileInfo: Fuse.FileInfo): MaybePromise<void>;
}

export function mountAndRun<FileHandle>(name: string, path: string, fs: FileSystem<FileHandle>) {
	const fileHandles = new Map<number, FileHandle>();
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
			create: (path, mode, fileInfo) => wrapErrnoCallback(
				`create(${path})`,
				async () => {
					const result = await fs.create(path, mode, fileInfo);
					if (result) {
						fileHandles.set(fileInfo.fh, result);
						return { fh: fileInfo.fh };
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
			open: (path, fileInfo) => wrapErrnoCallback(
				`open(${path})`,
				async () => {
					const result = await fs.open(path, fileInfo);
					if (result) {
						fileHandles.set(fileInfo.fh, result);
						return { fh: fileInfo.fh };
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
			read: (path, buffer, fileInfo) => wrapErrnoCallback(
				`read(${path})`,
				async () => {
					const fileHandle = fileHandles.get(fileInfo.fh);
					if (!fileHandle) {
						logger.error(`no open file handle for ${fileInfo.fh}`);
						throw new ErrnoException(Errno.ENOENT);
					}
					const result = await fs.read(path, buffer, fileHandle, fileInfo);
					if (typeof result === "number") {
						return result;
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
			write: (path, buffer, fileInfo) => wrapErrnoCallback(
				`write(${path})`,
				async () => {
					const fileHandle = fileHandles.get(fileInfo.fh);
					if (!fileHandle) {
						logger.error(`no open file handle for ${fileInfo.fh}`);
						throw new ErrnoException(Errno.ENOENT);
					}
					const result = fs.write(path, buffer, fileHandle, fileInfo);
					if (typeof result === "number") {
						return result;
					}
					throw new ErrnoException(Errno.ENOENT);
				}
			),
			unlink: (path) => wrapErrnoCallback(
				`unlink(${path})`,
				async () => {
					await fs.unlink(path);
				}
			),
			chmod: (path, mode) => wrapErrnoCallback(
				`chmod(${path})`,
				async () => {
					await fs.chmod(path, mode);
				}
			),
			chown: (path, user, group) => wrapErrnoCallback(
				`chown(${path})`,
				async () => {
					await fs.chown(path, user, group);
				}
			),
			release: (path, fileInfo) => wrapErrnoCallback(
				`release(${path})`,
				async () => {
					const fileHandle = fileHandles.get(fileInfo.fh);
					if (!fileHandle) {
						logger.error(`no open file handle for ${fileInfo.fh}`);
						throw new ErrnoException(Errno.ENOENT);
					}
					await fs.release(path, fileHandle, fileInfo);
				}
			),
		}
	);
}