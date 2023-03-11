import { Rgba, Size2 } from "../geometry";
import { AppState } from "../state-machine";

export class SolidColorState implements AppState<WebGL2RenderingContext> {
	private size = new Size2(0, 0);

	constructor(readonly color: Rgba) { }

	activate(_context: AppState.Context<WebGL2RenderingContext>): void {
		// nothing to do
	}

	deactivate(_context: AppState.Context<WebGL2RenderingContext>): void {
		// nothing to do	
	}

	resize(_context: AppState.Context<WebGL2RenderingContext>): void {
		// nothing to do
	}

	render(context: AppState.Context<WebGL2RenderingContext>): void {
		const gl = context.renderingContext;
		gl.viewport(0, 0, this.size.width, this.size.height);
		gl.clearColor(this.color.red, this.color.green, this.color.blue, this.color.alpha);
		gl.clear(gl.COLOR_BUFFER_BIT);
	}

	update(_context: AppState.Context<WebGL2RenderingContext>, _elapsedTime: number): AppState<WebGL2RenderingContext> | null | undefined {
		return null;
	}
}