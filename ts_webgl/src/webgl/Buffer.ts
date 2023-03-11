import { Disposable } from "../utils";

export class Buffer extends Disposable {
	private readonly buffer: WebGLBuffer;

	private byteLength_ = 0;

	constructor(
		private readonly gl: WebGL2RenderingContext,
		private readonly target: Buffer.Target,
	) {
		super();

		const buffer = gl.createBuffer();
		if (!buffer) {
			throw new Error("error creating buffer");
		}
		this.buffer = buffer;
	}

	get byteLength(): number {
		return this.byteLength_;
	}

	bind() {
		this.gl.bindBuffer(this.target, this.buffer);
	}

	bindNone() {
		this.gl.bindBuffer(this.target, null);
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
		this.bind();
		if (typeof sizeOrSrcData === "number") {
			this.gl.bufferData(this.target, sizeOrSrcData, usage);
			this.byteLength_ = sizeOrSrcData;
		} else if (srcOffset === undefined) {
			this.gl.bufferData(this.target, sizeOrSrcData, usage);
			this.byteLength_ = sizeOrSrcData?.byteLength ?? 0;
		} else {
			this.gl.bufferData(this.target, sizeOrSrcData as ArrayBufferView, usage, srcOffset, length);
			this.byteLength_ = length ?? sizeOrSrcData?.byteLength ?? 0;
		}
		this.bindNone();
	}

	subData(dstByteOffset: GLintptr, srcData: BufferSource): void;
	subData(dstByteOffset: GLintptr, srcData: ArrayBufferView, srcOffset: GLuint, length?: GLuint): void;
	subData(dstByteOffset: GLintptr, srcData: BufferSource | ArrayBufferView, srcOffset?: GLuint, length?: GLuint): void {
		if (srcOffset === undefined) {
			this.gl.bufferSubData(this.target, dstByteOffset, srcData);
		} else {
			this.gl.bufferSubData(this.target, dstByteOffset, srcData as ArrayBufferView, srcOffset, length);
		}
	}

	protected disposeImpl(): void {
		this.gl.deleteBuffer(this.buffer);
	}
}

export namespace Buffer {
	export enum Target {
		ARRAY = 0x8892,
		ELEMENT_ARRAY = 0x8893,
	}

	export enum Usage {
		STATIC_DRAW = 0x88E4,
		DYNAMIC_DRAW = 0x88E8,
		STREAM_DRAW = 0x88E0,
		STATIC_READ = 0x88E5,
		DYNAMIC_READ = 0x88E9,
		STREAM_READ = 0x88E1,
		STATIC_COPY = 0x88E6,
		DYNAMIC_COPY = 0x88EA,
		STREAM_COPY = 0x88E2,
	}
}
