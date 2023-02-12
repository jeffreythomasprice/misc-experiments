export class Logger {
	level: Logger.Level = Logger.Level.Info;

	error(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Error)) {
			console.error("%cERROR", "color:red", ...args);
		}
	}

	warn(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Warn)) {
			console.warn("%cWARN", "color:yellow", ...args);
		}
	}

	info(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Info)) {
			console.info("%cINFO", "color:green", ...args);
		}
	}

	debug(...args: unknown[]): void {
		if (this.isLevelEnabled(Logger.Level.Debug)) {
			console.debug("%cDEBUG", "color:purple", ...args);
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
