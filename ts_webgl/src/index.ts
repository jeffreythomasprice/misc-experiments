import shaderVertexSource from "bundle-text:./assets/shader.vertex";
import shaderFragmentSource from "bundle-text:./assets/shader.fragment";

import { Matrix4, Size2, Vector3 } from "./geometry";
import { AppState, run } from "./state-machine";
import { Shader, Buffer as WebGLBuffer, VertexArray, Texture2d } from "./webgl";

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

class DemoState implements AppState {
	private size = new Size2(0, 0);
	private orthoMatrix = Matrix4.identity;
	private perspectiveMatrix = Matrix4.identity;
	private shader?: Shader;
	private texture?: Texture2d;
	private arrayBuffer?: WebGLBuffer;
	private vertexArray?: VertexArray;

	private rotation = 0;

	activate(gl: WebGL2RenderingContext) {
		this.shader = new Shader(gl, shaderVertexSource, shaderFragmentSource);

		loadTextureFromURL(gl, new URL("./assets/bricks.png", import.meta.url))
			.then((texture) => {
				this.texture = texture;
			});

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

		const stride = Float32Array.BYTES_PER_ELEMENT * 9;
		const positionOffset = 0;
		const textureCoordinateOffset = Float32Array.BYTES_PER_ELEMENT * 3;
		const colorOffset = textureCoordinateOffset + Float32Array.BYTES_PER_ELEMENT * 2;
		this.vertexArray = new VertexArray(gl);
		this.vertexArray.bind();
		this.arrayBuffer.bind();
		const positionAttribute = this.shader.attributes.get("positionAttribute")!.location;
		gl.enableVertexAttribArray(positionAttribute);
		gl.vertexAttribPointer(positionAttribute, 2, gl.FLOAT, false, stride, positionOffset);
		const textureCoordinateAttribute = this.shader.attributes.get("textureCoordinateAttribute")!.location;
		gl.enableVertexAttribArray(textureCoordinateAttribute);
		gl.vertexAttribPointer(textureCoordinateAttribute, 2, gl.FLOAT, false, stride, textureCoordinateOffset);
		const colorAttribute = this.shader.attributes.get("colorAttribute")!.location;
		gl.enableVertexAttribArray(colorAttribute);
		gl.vertexAttribPointer(colorAttribute, 4, gl.FLOAT, false, stride, colorOffset);
		this.vertexArray.bindNone();
		this.arrayBuffer.bindNone();
		gl.disableVertexAttribArray(positionAttribute);
		gl.disableVertexAttribArray(textureCoordinateAttribute);
		gl.disableVertexAttribArray(colorAttribute);
	}

	deactivate(): void {
		this.shader?.dispose();
		this.arrayBuffer?.dispose();
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

run(new DemoState());
