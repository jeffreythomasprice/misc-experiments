import shaderVertexSource from "bundle-text:./assets/shader.vertex";
import shaderFragmentSource from "bundle-text:./assets/shader.fragment";

import { Aabb2, Matrix4, Rgba, Size2, Vector2, Vector3 } from "./geometry";
import { AppState, AsyncOperationState, run } from "./state-machine";
import { Shader, Texture2d, VertexDefinition, StructIO, Mesh } from "./webgl";
import { createOffscreenCanvasAndContext, getTextPlacement, loadFontFromURL, Logger, TextPlacement, wrap } from "./utils";

class Vertex {
	constructor(
		readonly position: Vector3,
		readonly textureCoordinate: Vector2,
		readonly color: Rgba,
	) { }
}

class VertexWriter extends StructIO<Vertex> {
	read(source: Buffer, offset: number): Vertex {
		const x = source.readFloatLE(offset);
		offset += 4;
		const y = source.readFloatLE(offset);
		offset += 4;
		const z = source.readFloatLE(offset);
		offset += 4;
		const tx = source.readFloatLE(offset);
		offset += 4;
		const ty = source.readFloatLE(offset);
		offset += 4;
		const r = source.readFloatLE(offset);
		offset += 4;
		const g = source.readFloatLE(offset);
		offset += 4;
		const b = source.readFloatLE(offset);
		offset += 4;
		const a = source.readFloatLE(offset);
		offset += 4;
		return new Vertex(
			new Vector3(x, y, z),
			new Vector2(tx, ty),
			new Rgba(r, g, b, a),
		);
	}

	write(destination: Buffer, offset: number, source: Vertex): void {
		offset = destination.writeFloatLE(source.position.x, offset);
		offset = destination.writeFloatLE(source.position.y, offset);
		offset = destination.writeFloatLE(source.position.z, offset);
		offset = destination.writeFloatLE(source.textureCoordinate.x, offset);
		offset = destination.writeFloatLE(source.textureCoordinate.y, offset);
		offset = destination.writeFloatLE(source.color.red, offset);
		offset = destination.writeFloatLE(source.color.green, offset);
		offset = destination.writeFloatLE(source.color.blue, offset);
		offset = destination.writeFloatLE(source.color.alpha, offset);
	}
}

class SolidColorState implements AppState {
	private size = new Size2(0, 0);

	constructor(readonly color: Rgba) { }

	activate(_gl: WebGL2RenderingContext): void {
		// nothing to do
	}

	deactivate(): void {
		// nothing to do
	}

	resize(size: Size2): void {
		this.size = size;
	}

	render(gl: WebGL2RenderingContext): void {
		gl.viewport(0, 0, this.size.width, this.size.height);
		gl.clearColor(this.color.red, this.color.green, this.color.blue, this.color.alpha);
		gl.clear(gl.COLOR_BUFFER_BIT);
	}

	update(_elapsedTime: number): AppState | null | undefined {
		return null;
	}
}

class DemoState implements AppState {
	private size = new Size2(0, 0);
	private orthoMatrix = Matrix4.identity;
	private perspectiveMatrix = Matrix4.identity;
	private shader?: Shader;
	private fontMesh?: Mesh<Vertex>;
	private imageMesh?: Mesh<Vertex>;

	private rotation = 0;
	private color = 0;

	constructor(
		private readonly fontTexture: Texture2d,
		private readonly imageTexture: Texture2d,
	) { }

	activate(gl: WebGL2RenderingContext): void {
		this.shader = new Shader(gl, shaderVertexSource, shaderFragmentSource);

		const vertexWriter = new VertexWriter(new VertexDefinition.Builder(gl, this.shader)
			.attribute("positionAttribute", (builder) => {
				builder
					.size(3)
					.type(WebGL2RenderingContext.FLOAT);
			})
			.attribute("textureCoordinateAttribute", (builder) => {
				builder
					.size(2)
					.type(WebGL2RenderingContext.FLOAT);
			})
			.attribute("colorAttribute", (builder) => {
				builder
					.size(4)
					.type(WebGL2RenderingContext.FLOAT);
			})
			.build());

		this.imageMesh = new Mesh(gl, vertexWriter);
		this.imageMesh.triangleFan(
			new Vertex(
				new Vector3(-1, -1, 0),
				new Vector2(0, 0),
				new Rgba(1, 1, 1, 1),
			),
			new Vertex(
				new Vector3(1, -1, 0),
				new Vector2(1, 0),
				new Rgba(1, 1, 1, 1),
			),
			new Vertex(
				new Vector3(1, 1, 0),
				new Vector2(1, 1),
				new Rgba(1, 1, 1, 1),
			),
			new Vertex(
				new Vector3(-1, 1, 0),
				new Vector2(0, 1),
				new Rgba(1, 1, 1, 1),
			),
		);

		this.fontMesh = new Mesh(gl, vertexWriter);
		this.fontMesh.triangleFan(
			new Vertex(
				new Vector3(0, 0, 0),
				new Vector2(0, 0),
				new Rgba(1, 1, 1, 1),
			),
			new Vertex(
				new Vector3(this.fontTexture.width, 0, 0),
				new Vector2(1, 0),
				new Rgba(1, 1, 1, 1),
			),
			new Vertex(
				new Vector3(this.fontTexture.width, this.fontTexture.height, 0),
				new Vector2(1, 1),
				new Rgba(1, 1, 1, 1),
			),
			new Vertex(
				new Vector3(0, this.fontTexture.height, 0),
				new Vector2(0, 1),
				new Rgba(1, 1, 1, 1),
			),
		);
	}

	deactivate(): void {
		this.shader?.dispose();
		this.imageMesh?.dispose();
	}

	resize(size: Size2): void {
		this.size = size;
		this.orthoMatrix = Matrix4.createOrtho(0, size.width, size.height, 0, -1, 1);
		this.perspectiveMatrix = Matrix4.createPerspective(60 * Math.PI / 180, size.width, size.height, 1, 1000);
	}

	render(gl: WebGL2RenderingContext): void {
		gl.viewport(0, 0, this.size.width, this.size.height);

		gl.clearColor(0.25, 0.5, 0.75, 1);
		gl.clear(gl.COLOR_BUFFER_BIT);

		if (this.shader) {
			this.shader.use();

			gl.activeTexture(gl.TEXTURE0);
			this.imageTexture.bind();
			gl.uniform1i(this.shader.uniforms.get("samplerUniform")!.location, 0);

			gl.uniformMatrix4fv(
				this.shader.uniforms.get("projectionMatrixUniform")!.location,
				false,
				this.perspectiveMatrix.toArray()
			);
			gl.uniformMatrix4fv(
				this.shader.uniforms.get("modelviewMatrixUniform")!.location,
				false,
				Matrix4.createLookAt(
					new Vector3(
						Math.cos(this.rotation) * 6,
						0,
						Math.sin(this.rotation) * 6,
					),
					new Vector3(0, 0, 0),
					new Vector3(0, 1, 0),
				)
					.toArray()
			);

			if (this.imageMesh) {
				this.imageMesh.bind();
				gl.drawElements(gl.TRIANGLES, this.imageMesh.elementArrayBuffer.size, gl.UNSIGNED_SHORT, 0);
				this.imageMesh.bindNone();
			}

			this.imageTexture.bindNone();

			this.fontTexture.bind();

			gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
			gl.enable(gl.BLEND);

			gl.uniformMatrix4fv(
				this.shader.uniforms.get("projectionMatrixUniform")!.location,
				false,
				this.orthoMatrix.toArray()
			);
			gl.uniformMatrix4fv(
				this.shader.uniforms.get("modelviewMatrixUniform")!.location,
				false,
				Matrix4.identity.toArray()
			);

			if (this.fontMesh) {
				this.fontMesh.bind();
				gl.drawElements(gl.TRIANGLES, this.fontMesh.elementArrayBuffer.size, gl.UNSIGNED_SHORT, 0);
				this.fontMesh.bindNone();
			}

			gl.disable(gl.BLEND);

			this.fontTexture.bindNone();

			this.shader.useNone();
		}
	}

	update(elapsedTime: number): AppState | null | undefined {
		this.rotation = wrap(this.rotation + 45 * Math.PI / 180 * elapsedTime, 0, Math.PI * 2);
		this.color = wrap(this.color + elapsedTime * 0.5, 0, 1);

		if (this.imageMesh) {
			const color1 = new Rgba(1, 0, 0, 1);
			const color2 = new Rgba(0, 0, 1, 1);
			const color = color1.toVector.mul(this.color).add(color2.toVector.mul(1 - this.color)).toRgba;
			for (let i = 0; i < this.imageMesh.arrayBuffer.size; i++) {
				const v = this.imageMesh.arrayBuffer.get(i);
				this.imageMesh.arrayBuffer.set(i, new Vertex(
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

run(new AsyncOperationState(
	new SolidColorState(new Rgba(0.25, 0.25, 0.25, 1)),
	async (gl) => {
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
