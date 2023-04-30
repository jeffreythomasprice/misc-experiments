import addon, { Errno, Fuse, MaybePromise } from "../build/Release/addon";

import { ErrnoException, wrapErrnoCallback } from "./errors";
import { getLogger } from "./logging";

const logger = getLogger("fs");

export interface FileSystem {
	init(connectionInfo: Fuse.ConnectionInfo): MaybePromise<void>;
	destroy(): MaybePromise<void>;
	getattr(path: string): MaybePromise<Fuse.Stat | undefined | null>;
	readdir(path: string): MaybePromise<Fuse.ReaddirResult[] | undefined | null>;
	open(path: string, fileInfo: Fuse.FileInfo): MaybePromise<Fuse.OpenResult | undefined | null>;
	read(path: string, buffer: Buffer, fileInfo: Fuse.FileInfo): MaybePromise<number | undefined | null>;
}

export function mountAndRun(name: string, path: string, fs: FileSystem) {
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