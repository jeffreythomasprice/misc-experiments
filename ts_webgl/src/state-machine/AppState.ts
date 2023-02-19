import { Size2 } from "../geometry";

export interface AppState<RenderingContextType extends WebGLRenderingContextBase, RendererType> {
	activate(context: AppState.Context<RenderingContextType, RendererType>): void;
	deactivate(context: AppState.Context<RenderingContextType, RendererType>): void;
	resize(context: AppState.Context<RenderingContextType, RendererType>): void;
	render(context: AppState.Context<RenderingContextType, RendererType>): void;
	/**
	 * @param elapsedTime the time since the last call to update, in seconds
	 */
	update(context: AppState.Context<RenderingContextType, RendererType>, elapsedTime: number): AppState<RenderingContextType, RendererType> | undefined | null;
}

export namespace AppState {
	export interface Context<RenderingContextType extends WebGLRenderingContextBase, RendererType> {
		get renderingContext(): RenderingContextType;
		get renderer(): RendererType;
		get size(): Size2;
	}
}