module "*/addon" {
	export enum LogLevel {
		FATAL = 1,
		ERROR = 2,
		WARN = 3,
		INFO = 4,
		DEBUG = 5,
		TRACE = 6,
	}

	export type LogCallback = (timestamp: number, level: LogLevel, message: string) => void;

	export interface InitOptions {
		// a callback to invoke for each log message
		log?: LogCallback;
		// how many log events to hold in memory before trying to emit them all at once to the callback
		// defaults to emitting immediately, i.e. 0
		maxBufferSize?: number;
	}

	export namespace Fuse {
		export interface ConnectionInfo {
			// TODO fields
		}
	}

	export interface MountAndRunCallbacks {
		init?: (connectionInfo: FuseConnectionInfo) => void;
		destroy?: () => void;
	}

	export interface FuseMount {
		// returns the result of the fuse_loop
		close(): Promise<number>;
	}

	export function init(options?: InitOptions): void;
	// emits log events before closing
	export function close(): Promise<void>;

	export function mountAndRun(args: string[], callbacks: MountAndRunCallbacks): Promise<FuseMount>;
}
