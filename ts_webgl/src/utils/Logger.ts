export class Logger {
	readonly prefix: string | undefined;
	readonly level: Logger.Level;

	private readonly errorPrefix: string;
	private readonly warnPrefix: string;
	private readonly infoPrefix: string;
	private readonly debugPrefix: string;

	static get defaultLevel(): Logger.Level {
		return defaultLevel_;
	}

	static set defaultLevel(level: Logger.Level) {
		defaultLevel_ = level;
	}

	constructor(options?: {
		prefix?: string;
		level?: Logger.Level;
	}) {
		this.prefix = options?.prefix;
		this.level = options?.level ?? Logger.defaultLevel;

		let prefix;
		if (this.prefix) {
			prefix = `${this.prefix} `;
		} else {
			prefix = "";
		}
		this.errorPrefix = `${prefix}%cERROR`;
		this.warnPrefix = `${prefix}%cWARN`;
		this.infoPrefix = `${prefix}%cINFO`;
		this.debugPrefix = `${prefix}%cDEBUG`;
	}

	error(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Error)) {
			console.error(this.errorPrefix, "color:red", ...args);
		}
	}

	warn(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Warn)) {
			console.warn(this.warnPrefix, "color:yellow", ...args);
		}
	}

	info(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Info)) {
			console.info(this.infoPrefix, "color:green", ...args);
		}
	}

	debug(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Debug)) {
			console.debug(this.debugPrefix, "color:purple", ...args);
		}
	}

	private isLevelEnabled(level: Logger.Level): boolean {
		return level <= this.level;
	}
}

export namespace Logger {
	export enum Level {
		Error = 0,
		Warn = 1,
		Info = 2,
		Debug = 3,
	}
}

let defaultLevel_ = Logger.Level.Info;
