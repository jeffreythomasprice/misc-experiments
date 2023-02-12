import shaderVertexSource from "bundle-text:./assets/shader.vertex";
import shaderFragmentSource from "bundle-text:./assets/shader.fragment";

import { Matrix4, Rgba, Size2, Vector2, Vector3, Vector4 } from "./geometry";
import { AppState, AsyncOperationState, run } from "./state-machine";
import { Shader, VertexArray, Texture2d, VertexDefinition, StructIO, ArrayBuffer, ElementArrayBuffer } from "./webgl";
import { Disposable, Logger } from "./utils";

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

// TODO move me
class Mesh<T> extends Disposable {
	readonly arrayBuffer: ArrayBuffer<T>;
	readonly elementArrayBuffer: ElementArrayBuffer;

	private readonly vertexArray: VertexArray;

	constructor(
		gl: WebGL2RenderingContext,
		writer: StructIO<T>,
	) {
		super();

		this.arrayBuffer = new ArrayBuffer(gl, writer);
		this.elementArrayBuffer = new ElementArrayBuffer(gl);
		this.vertexArray = new VertexArray(gl);

		this.vertexArray.bind();
		this.arrayBuffer.bind();
		this.elementArrayBuffer.bind();
		this.vertexArray.bindNone();
		this.arrayBuffer.bindNone();
		this.elementArrayBuffer.bindNone();
	}

	bind() {
		this.arrayBuffer.flush();
		this.elementArrayBuffer.flush();
		this.vertexArray.bind();
	}

	bindNone() {
		this.vertexArray.bindNone();
	}

	triangleFan(...vertices: T[]) {
		if (vertices.length < 3) {
			throw new Error("can't make a triangle fan with fewer than three vertices");
		}
		const first = this.arrayBuffer.size;
		this.arrayBuffer.push(...vertices);
		this.elementArrayBuffer.ensureCapacity((vertices.length - 2) * 3);
		for (let i = 1; i < vertices.length - 1; i++) {
			this.elementArrayBuffer.push(first, i, i + 1);
		}
	}

	protected disposeImpl(): void {
		this.arrayBuffer.dispose();
		this.elementArrayBuffer.dispose();
		this.vertexArray.dispose();
	}
}

// TODO move me
function clamp(value: number, min: number, max: number): number {
	if (value < min) {
		return min;
	}
	if (value > max) {
		return max;
	}
	return value;
}

// TODO move me
function wrap(value: number, min: number, max: number): number {
	return ((value - min) % (max - min)) + min;
}

class DemoState implements AppState {
	private size = new Size2(0, 0);
	private orthoMatrix = Matrix4.identity;
	private perspectiveMatrix = Matrix4.identity;
	private shader?: Shader;
	private mesh?: Mesh<Vertex>;

	private rotation = 0;
	private color = 0;

	constructor(
		private readonly texture: Texture2d,
	) { }

	activate(gl: WebGL2RenderingContext): void {
		this.shader = new Shader(gl, shaderVertexSource, shaderFragmentSource);

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

		this.mesh = new Mesh(gl, new VertexWriter(vertexDef));
		this.mesh.triangleFan(
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
	}

	deactivate(): void {
		this.shader?.dispose();
		this.mesh?.dispose();
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

		if (!this.shader || !this.mesh) {
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

		this.mesh.bind();
		gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);
		this.mesh.bindNone();

		this.texture.bindNone();

		this.shader.useNone();
	}

	update(elapsedTime: number): AppState | null | undefined {
		this.rotation = wrap(this.rotation + 45 * Math.PI / 180 * elapsedTime, 0, Math.PI * 2);
		this.color = wrap(this.color + elapsedTime * 0.1, 0, 1);

		if (this.mesh) {
			const color1 = new Rgba(1, 0, 0, 1);
			const color2 = new Rgba(0, 0, 1, 1);
			const color = color1.toVector.mul(this.color).add(color2.toVector.mul(1 - this.color)).toRgba;
			for (let i = 0; i < this.mesh.arrayBuffer.size; i++) {
				const v = this.mesh.arrayBuffer.get(i);
				this.mesh.arrayBuffer.set(i, new Vertex(
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

run(new AsyncOperationState(
	new SolidColorState(new Rgba(0.25, 0.25, 0.25, 1)),
	async (gl) => {
		const texture = await Texture2d.createFromURL(gl, new URL("./assets/bricks.png", import.meta.url));
		return new DemoState(texture);
	},
));
