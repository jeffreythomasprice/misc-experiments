import { LogLevel } from "../build/Release/addon";

export const levelsNames = Object.freeze({
	[LogLevel.FATAL]: "FATAL",
	[LogLevel.ERROR]: "ERROR",
	[LogLevel.WARN]: "WARN ",
	[LogLevel.INFO]: "INFO ",
	[LogLevel.DEBUG]: "DEBUG",
	[LogLevel.TRACE]: "TRACE",
});

export interface LoggerOptions {
	level?: LogLevel;
	prefix?: string;
	parent?: Logger;
}

export class Logger {
	readonly parent?: Logger;
	readonly prefix: string;
	private _level?: LogLevel;

	constructor(options?: LoggerOptions) {
		this.parent = options?.parent;
		this.prefix = options?.prefix ?? "";
		this._level = options?.level;
	}

	get level(): LogLevel {
		if (this._level) {
			return this._level;
		}
		if (this.parent) {
			return this.parent.level;
		}
		return LogLevel.INFO;
	}

	set level(level: LogLevel | undefined | null) {
		this._level = level || undefined;
	}

	log(timestamp: Date, level: LogLevel, message: string, ...params: unknown[]): void
	log(level: LogLevel, message: string, ...params: unknown[]): void
	log(...args: unknown[]): void {
		let timestamp: Date;
		let level: LogLevel;
		let message: string;
		let params: unknown[];
		if (args[0] instanceof Date) {
			timestamp = args[0] as Date;
			level = args[1] as LogLevel;
			message = args[2] as string;
			params = args.slice(3);
		} else {
			timestamp = new Date();
			level = args[0] as LogLevel;
			message = args[1] as string;
			params = args.slice(2);
		}
		if (level <= this.level) {
			const parts = [message, ...params];
			if (this.prefix) {
				parts.unshift(this.prefix);
			}
			log(
				timestamp,
				level,
				parts
					.map(p => {
						if (p instanceof Error) {
							return p.stack;
						}
						if (typeof (p as any).toString === "function") {
							return (p as any).toString();
						}
						return p;
					})
					.join(" ")
			);
		}
	}

	fatal(message: string, ...params: unknown[]) {
		this.log(LogLevel.FATAL, message, ...params);
	}

	error(message: string, ...params: unknown[]) {
		this.log(LogLevel.ERROR, message, ...params);
	}

	warn(message: string, ...params: unknown[]) {
		this.log(LogLevel.WARN, message, ...params);
	}

	info(message: string, ...params: unknown[]) {
		this.log(LogLevel.INFO, message, ...params);
	}

	debug(message: string, ...params: unknown[]) {
		this.log(LogLevel.DEBUG, message, ...params);
	}

	trace(message: string, ...params: unknown[]) {
		this.log(LogLevel.TRACE, message, ...params);
	}
}

const loggers = new Map<string, Logger>();

export function getLogger(name: string = ""): Logger {
	let result = loggers.get(name);
	if (!result) {
		if (name === "") {
			result = new Logger();
		} else {
			const parts = name.split(".");
			parts.pop();
			const parent = getLogger(parts.join("."));
			result = new Logger({
				parent,
				prefix: name
			});
		}
		loggers.set(name, result);
	}
	return result;
}

function log(timestamp: Date, level: LogLevel, message: string): void {
	const text = `${timestamp.toISOString()} [${levelsNames[level]}] ${message}`;
	if (level <= LogLevel.ERROR) {
		console.error(text);
	} else {
		console.log(text);
	}
}