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
		Array = 0x8892,
		ElementArray = 0x8893,
	}

	export enum Usage {
		StaticDraw = 0x88E4,
		DynamicDraw = 0x88E8,
		StreamDraw = 0x88E0,
		StaticRead = 0x88E5,
		DynamicRead = 0x88E9,
		StreamRead = 0x88E1,
		StaticCopy = 0x88E6,
		DynamicCopy = 0x88EA,
		StreamCopy = 0x88E2,
	}
}
