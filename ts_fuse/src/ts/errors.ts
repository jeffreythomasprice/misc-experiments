import { Errno, MaybePromise } from "../build/Release/addon";

import { getLogger } from "./logging";

const logger = getLogger();

export const errnoNames = Object.freeze({
	[Errno.EPERM]: "EPERM",
	[Errno.ENOENT]: "ENOENT",
	[Errno.ESRCH]: "ESRCH",
	[Errno.EINTR]: "EINTR",
	[Errno.EIO]: "EIO",
	[Errno.ENXIO]: "ENXIO",
	[Errno.E2BIG]: "E2BIG",
	[Errno.ENOEXEC]: "ENOEXEC",
	[Errno.EBADF]: "EBADF",
	[Errno.ECHILD]: "ECHILD",
	[Errno.EAGAIN]: "EAGAIN",
	[Errno.ENOMEM]: "ENOMEM",
	[Errno.EACCES]: "EACCES",
	[Errno.EFAULT]: "EFAULT",
	[Errno.ENOTBLK]: "ENOTBLK",
	[Errno.EBUSY]: "EBUSY",
	[Errno.EEXIST]: "EEXIST",
	[Errno.EXDEV]: "EXDEV",
	[Errno.ENODEV]: "ENODEV",
	[Errno.ENOTDIR]: "ENOTDIR",
	[Errno.EISDIR]: "EISDIR",
	[Errno.EINVAL]: "EINVAL",
	[Errno.ENFILE]: "ENFILE",
	[Errno.EMFILE]: "EMFILE",
	[Errno.ENOTTY]: "ENOTTY",
	[Errno.ETXTBSY]: "ETXTBSY",
	[Errno.EFBIG]: "EFBIG",
	[Errno.ENOSPC]: "ENOSPC",
	[Errno.ESPIPE]: "ESPIPE",
	[Errno.EROFS]: "EROFS",
	[Errno.EMLINK]: "EMLINK",
	[Errno.EPIPE]: "EPIPE",
	[Errno.EDOM]: "EDOM",
	[Errno.ERANGE]: "ERANGE",
});

function message(errno: Errno) {
	return errnoNames[errno] ?? errno.toString();
}

export class ErrnoException extends Error {
	constructor(public readonly errno: Errno) {
		super(`errno ${message(errno)}`);
	}
}

export async function wrapErrnoCallback<T>(functionDescription: string, f: () => MaybePromise<T>) {
	try {
		return await f();
	} catch (e) {
		if (e instanceof ErrnoException) {
			logger.error(`${functionDescription} ${e.message}`);
			return -e.errno;
		} else {
			logger.error(`${functionDescription} error`, e);
			return -Errno.EIO;
		}
	}
}