import { Disposable } from "../utils";

export class Buffer extends Disposable {
	private readonly glTarget: number;
	private readonly buffer: WebGLBuffer;

	constructor(
		private readonly gl: WebGL2RenderingContext,
		target: Buffer.Target,
	) {
		super();

		switch (target) {
			case Buffer.Target.Array:
				this.glTarget = gl.ARRAY_BUFFER;
				break;
			case Buffer.Target.ElementArray:
				this.glTarget = gl.ELEMENT_ARRAY_BUFFER;
				break;
			default:
				throw new Error(`unhandled buffer type: ${target}`);
		}

		const buffer = gl.createBuffer();
		if (!buffer) {
			throw new Error("error creating buffer");
		}
		this.buffer = buffer;
	}

	bind() {
		this.gl.bindBuffer(this.glTarget, this.buffer);
	}

	bindNone() {
		this.gl.bindBuffer(this.glTarget, null);
	}

	bufferData(size: GLsizeiptr, usage: Buffer.Usage): void;
	bufferData(srcData: BufferSource | null, usage: Buffer.Usage): void;
	bufferData(srcData: ArrayBufferView, usage: Buffer.Usage, srcOffset: GLuint, length?: GLuint): void;
	bufferData(
		sizeOrSrcData: GLsizeiptr | BufferSource | ArrayBufferView | null,
		usage: Buffer.Usage,
		srcOffset?: GLuint,
		length?: GLuint,
	): void {
		const glUsage = this.getGlUsage(usage);
		this.bind();
		if (typeof sizeOrSrcData === "number") {
			this.gl.bufferData(this.glTarget, sizeOrSrcData, glUsage);
		} else if (srcOffset === undefined) {
			this.gl.bufferData(this.glTarget, sizeOrSrcData, glUsage);
		} else {
			this.gl.bufferData(this.glTarget, sizeOrSrcData as ArrayBufferView, glUsage, srcOffset, length);
		}
		this.bindNone();
	}

	protected disposeImpl(): void {
		this.gl.deleteBuffer(this.buffer);
	}

	private getGlUsage(usage: Buffer.Usage) {
		switch (usage) {
			case Buffer.Usage.StaticDraw:
				return this.gl.STATIC_DRAW;
			case Buffer.Usage.DynamicDraw:
				return this.gl.DYNAMIC_DRAW;
			case Buffer.Usage.StreamDraw:
				return this.gl.STREAM_DRAW;
			case Buffer.Usage.StaticRead:
				return this.gl.STATIC_READ;
			case Buffer.Usage.DynamicRead:
				return this.gl.DYNAMIC_READ;
			case Buffer.Usage.StreamRead:
				return this.gl.STREAM_READ;
			case Buffer.Usage.StaticCopy:
				return this.gl.STATIC_COPY;
			case Buffer.Usage.DynamicCopy:
				return this.gl.DYNAMIC_COPY;
			case Buffer.Usage.StreamCopy:
				return this.gl.STREAM_COPY;
			default:
				throw new Error(`unrecognized usage: ${usage}`);
		}
	}
}

export namespace Buffer {
	export enum Target {
		Array,
		ElementArray
	}

	export enum Usage {
		StaticDraw,
		DynamicDraw,
		StreamDraw,
		StaticRead,
		DynamicRead,
		StreamRead,
		StaticCopy,
		DynamicCopy,
		StreamCopy
	}
}
