export * from "./AppState";

import { Size2 } from "../geometry";
import { awaitIfNeeded } from "../utils";
import { AppState } from "./AppState";

export function run(initialState: AppState) {
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
			await awaitIfNeeded(currentState.activate(gl));
		} catch (e) {
			// TODO logger
			console.error("error invoking activate on initial state", e);
		}

		let currentSize: Size2;
		const resize = async () => {
			try {
				currentSize = new Size2(window.innerWidth, window.innerHeight);
				canvas.width = currentSize.width;
				canvas.height = currentSize.height;
				await awaitIfNeeded(currentState.resize(currentSize));
			} catch (e) {
				// TODO logger
				console.error("error invoking resize on current state", e);
			}
		};
		window.addEventListener("resize", resize);
		await resize();

		const animate = async (time: number) => {
			try {
				currentState.render(gl);
			} catch (e) {
				// TODO logger
				console.error("error invoking render on current state", e);
			}
			let newState;
			try {
				newState = await awaitIfNeeded(currentState.update(time));
			} catch (e) {
				// TODO logger
				console.error("error invoking update on current state", e);
			}
			if (newState && newState !== currentState) {
				try {
					await awaitIfNeeded(newState.activate(gl));
				} catch (e) {
					// TODO logger
					console.error("error invoking activate on new state", e);
				}
				try {
					await awaitIfNeeded(currentState.deactivate());
				} catch (e) {
					// TODO logger
					console.error("error invoking deactivate on old state", e);
				}
				currentState = newState;
			}
			requestAnimationFrame(animate);
		};
		requestAnimationFrame(animate);
	});
}
