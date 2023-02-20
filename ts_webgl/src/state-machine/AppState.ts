import { Size2 } from "../geometry";

export interface AppState<RenderingContextType extends WebGLRenderingContextBase> {
	activate(context: AppState.Context<RenderingContextType>): void;
	deactivate(context: AppState.Context<RenderingContextType>): void;
	resize(context: AppState.Context<RenderingContextType>): void;
	render(context: AppState.Context<RenderingContextType>): void;
	/**
	 * @param elapsedTime the time since the last call to update, in seconds
	 */
	update(context: AppState.Context<RenderingContextType>, elapsedTime: number): AppState<RenderingContextType> | undefined | null;
}

export namespace AppState {
	export interface Context<RenderingContextType extends WebGLRenderingContextBase> {
		get renderingContext(): RenderingContextType;
		get size(): Size2;
	}
}