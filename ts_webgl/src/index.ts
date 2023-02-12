import shaderVertexSource from "bundle-text:./assets/shader.vertex";
import shaderFragmentSource from "bundle-text:./assets/shader.fragment";

import { Matrix4, Rgba, Size2, Vector3 } from "./geometry";
import { AppState, run } from "./state-machine";
import { Shader, Buffer as WebGLBuffer, VertexArray, Texture2d, VertexDefinition } from "./webgl";
import { Logger } from "./utils";

// TODO move me
function loadImageFromURL(url: URL): Promise<HTMLImageElement> {
	return new Promise((resolve, reject) => {
		const result = document.createElement("img");
		result.addEventListener("load", () => {
			resolve(result);
		});
		result.addEventListener("error", () => {
			reject(new Error(`failed to load from ${url}`));
		});
		result.src = url.toString();
	});
}

// TODO should be part of texture
async function loadTextureFromURL(gl: WebGL2RenderingContext, url: URL): Promise<Texture2d> {
	const image = await loadImageFromURL(url);
	const result = new Texture2d(gl);
	result.texImage(0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
	return result;
}

// TODO move me
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

// TODO move me
class AsyncOperationState implements AppState {
	private isActive = false;
	private nextState: AppState | null = null;

	private readonly logger: Logger;

	constructor(
		private readonly wrappedState: AppState,
		private readonly factory: (gl: WebGL2RenderingContext) => Promise<AppState>
	) {
		this.logger = new Logger({
			prefix: "async-op-state",
		});
	}

	activate(gl: WebGL2RenderingContext): void {
		this.wrappedState.activate(gl);
		this.isActive = true;
		this.factory(gl)
			.then((nextState) => {
				if (this.isActive) {
					this.logger.debug("pending operation completed, advancing states");
					this.nextState = nextState;
				} else {
					this.logger.warn("pending operation completed, but this state is no longer active, can't transition");
				}
			})
			.catch((e) => {
				this.logger.error("pending operation failed", e);
			});
	}

	deactivate(): void {
		this.wrappedState.deactivate();
		this.isActive = false;
		this.nextState = null;
	}

	resize(size: Size2): void {
		this.wrappedState.resize(size);
	}

	render(gl: WebGL2RenderingContext): void {
		this.wrappedState.render(gl);
	}

	update(elapsedTime: number): AppState | null | undefined {
		const next = this.wrappedState.update(elapsedTime);
		if (next) {
			this.logger.debug("while waiting for pending operation to complete the wrapped state signaled it wants to transition to a new state, using that instead");
			return next;
		}
		if (this.nextState) {
			this.logger.debug("transitioning to the result of the pending operation");
			return this.nextState;
		}
		return null;
	}
}

class DemoState implements AppState {
	private size = new Size2(0, 0);
	private orthoMatrix = Matrix4.identity;
	private perspectiveMatrix = Matrix4.identity;
	private shader?: Shader;
	private arrayBuffer?: WebGLBuffer;
	private vertexArray?: VertexArray;

	private rotation = 0;

	constructor(
		private readonly texture: Texture2d,
	) { }

	activate(gl: WebGL2RenderingContext): void {
		this.shader = new Shader(gl, shaderVertexSource, shaderFragmentSource);

		this.arrayBuffer = new WebGLBuffer(gl, WebGLBuffer.Target.Array);
		this.arrayBuffer.bufferData(
			new Float32Array([
				-1, -1, 0,
				0, 1,
				1, 1, 1, 1,

				1, -1, 0,
				1, 1,
				1, 1, 1, 1,

				1, 1, 0,
				1, 0,
				1, 1, 1, 1,

				1, 1, 0,
				1, 0,
				1, 1, 1, 1,

				-1, 1, 0,
				0, 0,
				1, 1, 1, 1,

				-1, -1, 0,
				0, 1,
				1, 1, 1, 1,
			]),
			WebGLBuffer.Usage.StaticDraw
		);

		const vertexDef = new VertexDefinition.Builder(gl, this.shader)
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
			.build();

		// TODO mesh helper
		this.vertexArray = new VertexArray(gl);
		this.vertexArray.bind();
		this.arrayBuffer.bind();
		vertexDef.enable();
		this.vertexArray.bindNone();
		this.arrayBuffer.bindNone();
		vertexDef.disable();
	}

	deactivate(): void {
		this.shader?.dispose();
		this.arrayBuffer?.dispose();
		this.vertexArray?.dispose();
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

		if (!this.shader || !this.texture || !this.vertexArray) {
			return;
		}

		this.shader.use();

		gl.activeTexture(gl.TEXTURE0);
		this.texture.bind();
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

		this.vertexArray.bind();
		gl.drawArrays(gl.TRIANGLES, 0, 6);
		this.vertexArray.bindNone();

		this.texture.bindNone();

		this.shader.useNone();
	}

	update(elapsedTime: number): AppState | null | undefined {
		this.rotation = (this.rotation + 45 * Math.PI / 180 * elapsedTime) % (Math.PI * 2);
		return null;
	}
}

Logger.defaultLevel = Logger.Level.Debug;

run(new AsyncOperationState(
	new SolidColorState(new Rgba(0.25, 0.25, 0.25, 1)),
	async (gl) => {
		const texture = await loadTextureFromURL(gl, new URL("./assets/bricks.png", import.meta.url));
		return new DemoState(texture);
	},
));
