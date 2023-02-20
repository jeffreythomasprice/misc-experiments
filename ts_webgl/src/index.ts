import { Aabb2, Rgba, Size2, Vector2 } from "./geometry";
import { AppState, AsyncOperationState, run } from "./state-machine";
import { Shader, Texture2d } from "./webgl";
import { createOffscreenCanvasAndContext, getTextPlacement, loadFontFromURL, Logger, TextPlacement, wrap } from "./utils";
import { SolidColorState } from "./demo";
import { GraphicsState } from "./demo/GraphicsState";

class DemoState implements AppState<WebGL2RenderingContext> {
	private graphicsState?: GraphicsState;
	private shader?: Shader;
	private fontMesh?: GraphicsState.TexturedMesh;
	private imageMesh?: GraphicsState.TexturedMesh;

	private rotation = 0;
	private color = 0;

	constructor(
		private readonly fontTexture: Texture2d,
		private readonly imageTexture: Texture2d,
	) { }

	activate(context: AppState.Context<WebGL2RenderingContext>): void {
		const gl = context.renderingContext;

		this.graphicsState = new GraphicsState(gl);

		this.imageMesh = this.graphicsState.createTexturedMesh();
		this.imageMesh.triangleFan(
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(0, 0),
				new Vector2(0, 0),
				new Rgba(1, 1, 1, 1),
			),
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(this.imageTexture.width, 0),
				new Vector2(1, 0),
				new Rgba(1, 1, 1, 1),
			),
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(this.imageTexture.width, this.imageTexture.height),
				new Vector2(1, 1),
				new Rgba(1, 1, 1, 1),
			),
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(0, this.imageTexture.height),
				new Vector2(0, 1),
				new Rgba(1, 1, 1, 1),
			),
		);

		this.fontMesh = this.graphicsState.createTexturedMesh();
		this.fontMesh.triangleFan(
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(0, 0),
				new Vector2(0, 0),
				new Rgba(1, 1, 1, 1),
			),
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(this.fontTexture.width, 0),
				new Vector2(1, 0),
				new Rgba(1, 1, 1, 1),
			),
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(this.fontTexture.width, this.fontTexture.height),
				new Vector2(1, 1),
				new Rgba(1, 1, 1, 1),
			),
			new GraphicsState.VertexPos2CoordRgba(
				new Vector2(0, this.fontTexture.height),
				new Vector2(0, 1),
				new Rgba(1, 1, 1, 1),
			),
		);
	}

	deactivate(_context: AppState.Context<WebGL2RenderingContext>): void {
		this.shader?.dispose();
		this.imageMesh?.dispose();
	}

	resize(context: AppState.Context<WebGL2RenderingContext>): void {
		if (this.graphicsState) {
			this.graphicsState.projectionSize = context.size;
		}
	}

	render(context: AppState.Context<WebGL2RenderingContext>): void {
		const gl = context.renderingContext;

		gl.viewport(0, 0, context.size.width, context.size.height);

		gl.clearColor(0.25, 0.5, 0.75, 1);
		gl.clear(gl.COLOR_BUFFER_BIT);

		if (this.graphicsState) {
			this.graphicsState.begin();

			this.graphicsState.texture = this.imageTexture;
			if (this.imageMesh) {
				this.graphicsState.render(this.imageMesh, 0, this.imageMesh.elementArrayBuffer.size);
			}

			this.graphicsState.transparency = true;
			this.graphicsState.texture = this.fontTexture;
			if (this.fontMesh) {
				this.graphicsState.render(this.fontMesh, 0, this.fontMesh.elementArrayBuffer.size);
			}
			this.graphicsState.transparency = false;

			this.graphicsState.end();
		}
	}

	update(_context: AppState.Context<WebGL2RenderingContext>, elapsedTime: number): AppState<WebGL2RenderingContext> | null | undefined {
		this.rotation = wrap(this.rotation + 45 * Math.PI / 180 * elapsedTime, 0, Math.PI * 2);
		this.color = wrap(this.color + elapsedTime * 0.5, 0, 1);

		if (this.imageMesh) {
			const color1 = new Rgba(1, 0, 0, 1);
			const color2 = new Rgba(0, 0, 1, 1);
			const color = color1.toVector.mul(this.color).add(color2.toVector.mul(1 - this.color)).toRgba;
			for (let i = 0; i < this.imageMesh.arrayBuffer.size; i++) {
				const v = this.imageMesh.arrayBuffer.get(i);
				this.imageMesh.arrayBuffer.set(i, new GraphicsState.VertexPos2CoordRgba(
					v.position,
					v.textureCoordinate,
					color
				));
			}
		}

		return null;
	}
}

Logger.defaultLevel = Logger.Level.Debug;

// TODO make something reusable out of workers
const logger = new Logger();
const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
worker.addEventListener("message", (e) => {
	logger.debug("message from worker", e.data);
});
worker.addEventListener("messageerror", (e) => {
	logger.error("messageerror from worker", e);
});
worker.addEventListener("error", (e) => {
	logger.error("error from worker", e);
});
worker.postMessage("this came from index");

run(
	"webgl2",
	new AsyncOperationState(
		new SolidColorState(new Rgba(0.25, 0.25, 0.25, 1)),
		async (context) => {
			const gl = context.renderingContext as WebGL2RenderingContext;

			const font = await loadFontFromURL("custom-font", new URL("./assets/RobotoSlab-VariableFont_wght.ttf", import.meta.url));
			const textImage = createTestStringImage(
				getTextPlacement(
					// font weight, then size, then family
					// https://developer.mozilla.org/en-US/docs/Web/CSS/font
					`500 40px "${font.family}"`,
					"Hello, World!\n\njqyp\nHow does this handle big multi-line\ntext?",
					{
						bounds: new Aabb2(new Vector2(0, 0), new Size2(400, 800)),
						horizontal: "center",
						vertical: "center",
					}
				),
				"white",
				"black",
			);
			const textTexture = new Texture2d(gl);
			textTexture.texImage(0, Texture2d.Format.RGBA, Texture2d.Format.RGBA, Texture2d.Type.UNSIGNED_BYTE, textImage);

			const imageTexture = await Texture2d.createFromURL(gl, new URL("./assets/bricks.png", import.meta.url));
			return new DemoState(textTexture, imageTexture);
		},
	));

function createTestStringImage(
	metrics: TextPlacement,
	fillStyle: string | undefined | null,
	strokeStyle: string | undefined | null,
): OffscreenCanvas {
	const [result, context] = createOffscreenCanvasAndContext(new Size2(Math.ceil(metrics.bounds.width), Math.ceil(metrics.bounds.height)));
	context.font = metrics.font;
	if (fillStyle) {
		context.fillStyle = fillStyle;
	}
	if (strokeStyle) {
		context.strokeStyle = strokeStyle;
	}
	for (const lineMetrics of metrics.lines) {
		const p = lineMetrics.bounds.min.sub(metrics.bounds.min).add(new Vector2(0, lineMetrics.ascent));
		if (fillStyle) {
			context.fillText(lineMetrics.text, p.x, p.y);
		}
		if (strokeStyle) {
			context.strokeText(lineMetrics.text, p.x, p.y);
		}
	}
	return result;
}
