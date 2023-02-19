export * from "./AppState";
export * from "./AsyncOperationState";

import { Size2 } from "../geometry";
import { Logger } from "../utils";
import { AppState } from "./AppState";

class Context<RenderingContextType extends WebGLRenderingContextBase, RendererType> implements AppState.Context<RenderingContextType, RendererType> {
	constructor(
		private readonly _renderingContext: RenderingContextType,
		private readonly _renderer: RendererType,
		private _size: Size2,
	) { }

	get renderingContext(): RenderingContextType {
		return this._renderingContext;
	}

	get renderer(): RendererType {
		return this._renderer;
	}

	get size(): Size2 {
		return this._size;
	}

	set size(s: Size2) {
		this._size = s;
	}
}

export function run<RendererType>(
	renderingContextType: "webgl",
	rendererFactory: (context: WebGLRenderingContext) => RendererType,
	initialState: AppState<WebGLRenderingContext, RendererType>,
): void;
export function run<RendererType>(
	renderingContextType: "webgl2",
	rendererFactory: (context: WebGL2RenderingContext) => RendererType,
	initialState: AppState<WebGL2RenderingContext, RendererType>,
): void;
export function run<RendererType>(
	renderingContextType: "webgl" | "webgl2",
	rendererFactory: ((context: WebGLRenderingContext) => RendererType) | ((context: WebGL2RenderingContext) => RendererType),
	initialState: AppState<WebGLRenderingContextBase, RendererType>,
): void {
	const logger = new Logger({
		prefix: "state-machine",
	});

	window.addEventListener("DOMContentLoaded", async () => {
		const canvas = document.createElement("canvas");
		canvas.style.position = "absolute";
		canvas.style.top = "0";
		canvas.style.left = "0";
		canvas.style.width = "100%";
		canvas.style.height = "100%";
		document.body.replaceChildren(canvas);

		const renderingContext = canvas.getContext(
			renderingContextType,
			{
				powerPreference: "high-performance",
				alpha: true,
				depth: true,
				antialias: true
			}
		) as WebGLRenderingContext | WebGL2RenderingContext | null;
		if (!renderingContext) {
			throw new Error("failed to make webgl context");
		}

		const renderer = (rendererFactory as ((context: WebGLRenderingContextBase) => RendererType))(renderingContext);

		const context = new Context(renderingContext, renderer, new Size2(0, 0));

		let currentState = initialState;
		try {
			currentState.activate(context);
		} catch (e) {
			logger.error("error invoking activate on initial state", e);
		}

		let currentSize: Size2;
		const resize = () => {
			try {
				currentSize = new Size2(window.innerWidth, window.innerHeight);
				canvas.width = currentSize.width;
				canvas.height = currentSize.height;
				context.size = currentSize;
				currentState.resize(context);
			} catch (e) {
				logger.error("error invoking resize on current state", e);
			}
		};
		window.addEventListener("resize", resize);
		resize();

		let lastTime: number | null = null;
		const animate = (time: number) => {
			try {
				currentState.render(context);
			} catch (e) {
				logger.error("error invoking render on current state", e);
			}

			if (lastTime) {
				const elapsedTime = (time - lastTime) / 1000;
				let newState;
				try {
					newState = currentState.update(context, elapsedTime);
				} catch (e) {
					logger.error("error invoking update on current state", e);
				}

				if (newState && newState !== currentState) {
					try {
						newState.activate(context);
					} catch (e) {
						logger.error("error invoking activate on new state", e);
					}
					try {
						currentState.deactivate(context);
					} catch (e) {
						logger.error("error invoking deactivate on old state", e);
					}
					currentState = newState;
					resize();
				}
			}
			lastTime = time;

			requestAnimationFrame(animate);
		};
		requestAnimationFrame(animate);
	});
}
