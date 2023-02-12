import { Size2 } from "../geometry";
import { Logger } from "../utils";
import { AppState } from "./AppState";

export class AsyncOperationState implements AppState {
	private isActive = false;
	private nextState: AppState | null = null;

	private readonly logger: Logger;

	constructor(
		private readonly wrappedState: AppState,
		private readonly factory: (gl: WebGL2RenderingContext) => Promise<AppState>
	) {
		this.logger = new Logger({
			prefix: "async-op-state",
		});
	}

	activate(gl: WebGL2RenderingContext): void {
		this.wrappedState.activate(gl);
		this.isActive = true;
		this.factory(gl)
			.then((nextState) => {
				if (this.isActive) {
					this.logger.debug("pending operation completed, advancing states");
					this.nextState = nextState;
				} else {
					this.logger.warn("pending operation completed, but this state is no longer active, can't transition");
				}
			})
			.catch((e) => {
				this.logger.error("pending operation failed", e);
			});
	}

	deactivate(): void {
		this.wrappedState.deactivate();
		this.isActive = false;
		this.nextState = null;
	}

	resize(size: Size2): void {
		this.wrappedState.resize(size);
	}

	render(gl: WebGL2RenderingContext): void {
		this.wrappedState.render(gl);
	}

	update(elapsedTime: number): AppState | null | undefined {
		const next = this.wrappedState.update(elapsedTime);
		if (next) {
			this.logger.debug("while waiting for pending operation to complete the wrapped state signaled it wants to transition to a new state, using that instead");
			return next;
		}
		if (this.nextState) {
			this.logger.debug("transitioning to the result of the pending operation");
			return this.nextState;
		}
		return null;
	}
}