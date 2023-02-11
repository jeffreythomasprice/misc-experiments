import { Disposable } from "../utils";

export class VertexArray extends Disposable {
	private readonly vertexArray: WebGLVertexArrayObject;

	constructor(
		private readonly gl: WebGL2RenderingContext,
	) {
		super();

		const vertexArray = gl.createVertexArray();
		if (!vertexArray) {
			throw new Error("error creating vertex array");
		}

		this.vertexArray = vertexArray;
	}

	bind() {
		this.gl.bindVertexArray(this.vertexArray);
	}

	bindNone() {
		this.gl.bindVertexArray(null);
	}

	protected disposeImpl(): void {
		this.gl.deleteVertexArray(this.vertexArray);
	}
}