import { DirtyRegionTracker, Disposable } from "../utils";
import { StructWriter } from "./StructWriter";
import { Buffer as WebGLBuffer } from "./Buffer";

export class ArrayBuffer<T> extends Disposable {
	private readonly glBuffer: WebGLBuffer;
	private localBuffer: Buffer;

	private size_ = 0;
	private dirty = new DirtyRegionTracker();

	constructor(
		gl: WebGL2RenderingContext,
		readonly writer: StructWriter<T>,
	) {
		super();

		this.glBuffer = new WebGLBuffer(gl, WebGLBuffer.Target.Array);
		this.localBuffer = Buffer.allocUnsafe(0);
	}

	get stride(): number {
		return this.writer.def.stride;
	}

	/**
	 * number of vertices, not number of bytes
	 */
	get size(): number {
		return this.size_;
	}

	/**
	 * number of vertices this can store without resizing
	 */
	get capacity(): number {
		return this.localBuffer.byteLength / this.stride;
	}

	ensureCapacity(desiredCapacity: number) {
		if (desiredCapacity <= this.capacity) {
			return;
		}
		const newBuffer = Buffer.allocUnsafe(desiredCapacity * this.stride);
		this.localBuffer.copy(newBuffer);
		this.localBuffer = newBuffer;
	}

	push(...vertices: T[]): void {
		if (vertices.length === 0) {
			return;
		}
		this.dirty.add(
			this.size,
			this.size + vertices.length - 1
		);
		this.ensureCapacity(this.size + vertices.length);
		this.writer.update(
			this.localBuffer,
			this.size * this.stride,
			...vertices
		);
		this.size_ += vertices.length;
	}

	flush() {
		const range = this.dirty.clear();
		if (!range) {
			return;
		}
		if (this.glBuffer.byteLength !== this.localBuffer.byteLength) {
			this.glBuffer.bufferData(this.localBuffer, WebGLBuffer.Usage.DynamicDraw);
		} else {
			const offset = range.min * this.stride;
			const length = (range.max - range.min + 1) * this.stride;
			this.glBuffer.subData(offset, this.localBuffer, offset, length);
		}
	}

	bind() {
		if (this.dirty) {
			this.flush();
		}
		this.glBuffer.bind();
		this.writer.def.enable();
	}

	bindNone() {
		this.writer.def.disable();
		this.glBuffer.bindNone();
	}

	protected disposeImpl(): void {
		this.glBuffer.dispose();
	}
}