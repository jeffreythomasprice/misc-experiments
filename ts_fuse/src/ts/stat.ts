import {
	FileType,
	Fuse
} from "../build/Release/addon";


interface StatOptionsBase {
	// st_mode
	mode: FileType;
	// st_nlink
	linkCount?: number;
	// st_atim
	lastAccessTime: Date;
	// st_mtim
	modificationTime: Date;
	// st_ctim
	statusChangeTime: Date;
}

interface FileStatOptions extends StatOptionsBase {
	// st_size
	size: number;
}

interface DirectoryStatOptions extends StatOptionsBase { }

const zeroStat: Fuse.Stat = {
	st_mode: 0,
	st_nlink: 2,
	st_atim: { tv_sec: 0, tv_nsec: 0 },
	st_mtim: { tv_sec: 0, tv_nsec: 0 },
	st_ctim: { tv_sec: 0, tv_nsec: 0 },
	st_dev: 0,
	st_ino: 0,
	st_uid: 0,
	st_gid: 0,
	st_rdev: 0,
	st_size: 0,
	st_blksize: 0,
	st_blocks: 0,
};

export function fileStat(options: FileStatOptions): Fuse.Stat {
	return {
		...stat(options),
		...	{
			st_size: options.size,
		}
	};
}

export function directoryStat(options: DirectoryStatOptions): Fuse.Stat {
	return stat(options);
}

function stat(options: StatOptionsBase): Fuse.Stat {
	return {
		...zeroStat,
		...	{
			st_mode: options.mode,
			st_nlink: options.linkCount ?? 1,
			st_atim: dateToTimespec(options.lastAccessTime),
			st_mtim: dateToTimespec(options.modificationTime),
			st_ctim: dateToTimespec(options.statusChangeTime),
		}
	};
}

function dateToTimespec(date: Date): Fuse.Timespec {
	const milliseconds = date.valueOf();
	const seconds = Math.floor(milliseconds / 1000);
	return {
		tv_sec: seconds,
		tv_nsec: (milliseconds - seconds * 1000) * 1000000,
	};
}