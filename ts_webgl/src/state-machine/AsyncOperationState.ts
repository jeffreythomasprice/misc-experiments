import { Logger } from "../utils";
import { AppState } from "./AppState";

export class AsyncOperationState<RenderingContextType extends WebGLRenderingContextBase, RendererType> implements AppState<RenderingContextType, RendererType> {
	private isActive = false;
	private nextState: AppState<RenderingContextType, RendererType> | null = null;

	private readonly logger: Logger;

	constructor(
		private readonly wrappedState: AppState<RenderingContextType, RendererType>,
		private readonly factory: (context: AppState.Context<RenderingContextType, RendererType>) => Promise<AppState<RenderingContextType, RendererType>>
	) {
		this.logger = new Logger({
			prefix: "async-op-state",
		});
	}

	activate(context: AppState.Context<RenderingContextType, RendererType>): void {
		this.wrappedState.activate(context);
		this.isActive = true;
		this.factory(context)
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

	deactivate(context: AppState.Context<RenderingContextType, RendererType>): void {
		this.wrappedState.deactivate(context);
		this.isActive = false;
		this.nextState = null;
	}

	resize(context: AppState.Context<RenderingContextType, RendererType>): void {
		this.wrappedState.resize(context);
	}

	render(context: AppState.Context<RenderingContextType, RendererType>): void {
		this.wrappedState.render(context);
	}

	update(context: AppState.Context<RenderingContextType, RendererType>, elapsedTime: number): AppState<RenderingContextType, RendererType> | null | undefined {
		const next = this.wrappedState.update(context, elapsedTime);
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