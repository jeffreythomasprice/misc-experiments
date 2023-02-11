import { Size2 } from "../geometry";
import { MaybePromise } from "../utils/promises";

export interface AppState {
	activate(gl: WebGL2RenderingContext): MaybePromise<void>;
	deactivate(): MaybePromise<void>;
	resize(size: Size2): MaybePromise<void>;
	render(gl: WebGL2RenderingContext): MaybePromise<void>;
	update(time: number): MaybePromise<AppState | undefined | null>;
}