import { Size2 } from "../geometry";

export interface AppState {
	activate(gl: WebGL2RenderingContext): void;
	deactivate(): void;
	resize(size: Size2): void;
	render(gl: WebGL2RenderingContext): void;
	/**
	 * @param elapsedTime the time since the last call to update, in seconds
	 */
	update(elapsedTime: number): AppState | undefined | null;
}