import addon, { LogLevel } from "../build/Release/addon";

const levelsNames = Object.freeze({
	[LogLevel.FATAL]: "FATAL",
	[LogLevel.ERROR]: "ERROR",
	[LogLevel.WARN]: "WARN ",
	[LogLevel.INFO]: "INFO ",
	[LogLevel.DEBUG]: "DEBUG",
	[LogLevel.TRACE]: "TRACE",
});


function log(timestamp: Date, level: LogLevel, message: string): void {
	const text = `${timestamp.toISOString()} [${levelsNames[level]}] ${message}`;
	if (level <= LogLevel.ERROR) {
		console.error(text);
	} else {
		console.log(text);
	}
}

class Logger {
	level = LogLevel.INFO;
	readonly prefix: string;

	constructor(options?: {
		level?: LogLevel;
		prefix?: string;
	}) {
		this.level = options?.level ?? LogLevel.INFO;
		this.prefix = options?.prefix ?? "";
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

const logger = new Logger({
	level: LogLevel.DEBUG
});

(async () => {
	// TODO support child loggers
	const addonLogger = new Logger({
		level: LogLevel.DEBUG,
		prefix: "c++"
	});

	addon.init({
		log: (timestamp, level, message) => {
			addonLogger.log(new Date(timestamp), level, message);
		},
	});

	const mount = await addon.mountAndRun(
		[
			"experiment",
			"/home/jeff/mount_points/test",
			// foreground mode
			"-f",
		],
		{
			init: (connectionInfo) => {
				logger.debug(`init ${JSON.stringify(connectionInfo)}`);
				// return Promise.resolve(42);
				return new Promise((resolve) => {
					setTimeout(
						() => {
							logger.debug("TODO simulated delay done, init complete");
							resolve(42);
						},
						1000
					);
				});
			},
			destroy: () => {
				logger.debug("destroy");
				return Promise.resolve();
			},
		}
	);
	logger.debug("mounted");

	await new Promise<void>((resolve) => {
		process.once("SIGINT", () => {
			resolve();
		});
	});

	const mountResult = await mount.close();
	logger.debug(`unmounted, result = ${mountResult}`);

	await addon.close();
	process.exit(0);
})()
	.catch((e) => {
		logger.fatal("error", e);
		process.exit(1);
	});
