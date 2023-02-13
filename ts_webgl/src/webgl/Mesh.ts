import { Disposable } from "../utils";
import { ArrayBuffer } from "./ArrayBuffer";
import { ElementArrayBuffer } from "./ElementArrayBuffer";
import { StructIO } from "./StructIO";
import { VertexArray } from "./VertexArray";

export class Mesh<T> extends Disposable {
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