export * from "./AppState";
export * from "./AsyncOperationState";

import { Size2 } from "../geometry";
import { Logger } from "../utils";
import { AppState } from "./AppState";

export function run(initialState: AppState) {
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

		const gl = canvas.getContext(
			"webgl2",
			{
				powerPreference: "high-performance",
				alpha: true,
				depth: true,
				antialias: true
			}
		);
		if (!gl) {
			throw new Error("failed to make webgl context");
		}

		let currentState = initialState;
		try {
			currentState.activate(gl);
		} catch (e) {
			logger.error("error invoking activate on initial state", e);
		}

		let currentSize: Size2;
		const resize = () => {
			try {
				currentSize = new Size2(window.innerWidth, window.innerHeight);
				canvas.width = currentSize.width;
				canvas.height = currentSize.height;
				currentState.resize(currentSize);
			} catch (e) {
				logger.error("error invoking resize on current state", e);
			}
		};
		window.addEventListener("resize", resize);
		resize();

		let lastTime: number | null = null;
		const animate = (time: number) => {
			try {
				currentState.render(gl);
			} catch (e) {
				logger.error("error invoking render on current state", e);
			}

			if (lastTime) {
				const elapsedTime = (time - lastTime) / 1000;
				let newState;
				try {
					newState = currentState.update(elapsedTime);
				} catch (e) {
					logger.error("error invoking update on current state", e);
				}

				if (newState && newState !== currentState) {
					try {
						newState.activate(gl);
					} catch (e) {
						logger.error("error invoking activate on new state", e);
					}
					try {
						currentState.deactivate();
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
