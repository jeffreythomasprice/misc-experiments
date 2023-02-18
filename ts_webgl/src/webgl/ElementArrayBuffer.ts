import { DirtyRegionTracker, Disposable } from "../utils";
import { Buffer as WebGLBuffer } from "./Buffer";

export class ElementArrayBuffer extends Disposable {
	private readonly glBuffer: WebGLBuffer;
	private localBuffer: Buffer;

	private size_ = 0;
	private dirty = new DirtyRegionTracker();

	constructor(
		gl: WebGL2RenderingContext,
	) {
		super();

		this.glBuffer = new WebGLBuffer(gl, WebGLBuffer.Target.ELEMENT_ARRAY);
		this.localBuffer = Buffer.allocUnsafe(0);
	}

	get stride(): number {
		return Uint16Array.BYTES_PER_ELEMENT;
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
		return this.localBuffer.byteLength / this.stride
	}

	ensureCapacity(desiredCapacity: number) {
		if (desiredCapacity <= this.capacity) {
			return;
		}
		const newBuffer = Buffer.allocUnsafe(desiredCapacity * this.stride);
		this.localBuffer.copy(newBuffer);
		this.localBuffer = newBuffer;
	}

	get(index: number): number {
		if (index < 0 || index >= this.size) {
			throw new Error(`index ${index} out of bounds, size ${this.size}`);
		}
		return new Uint16Array(this.localBuffer.buffer)[index];
	}

	set(index: number, value: number): void {
		if (index < 0 || index >= this.size) {
			throw new Error(`index ${index} out of bounds, size ${this.size}`);
		}
		new Uint16Array(this.localBuffer.buffer)[index] = value;
		this.dirty.add(index);
	}

	push(...indices: number[]): void {
		if (indices.length === 0) {
			return;
		}
		this.dirty.add(
			this.size,
			this.size + indices.length - 1
		);
		this.ensureCapacity(this.size + indices.length);
		new Uint16Array(this.localBuffer.buffer).set(indices, this.size);
		this.size_ += indices.length;
	}

	flush() {
		this.glBuffer.bind();
		const range = this.dirty.clear();
		if (!range) {
			return;
		}
		if (this.glBuffer.byteLength !== this.localBuffer.byteLength) {
			this.glBuffer.bufferData(this.localBuffer, WebGLBuffer.Usage.DYNAMIC_DRAW);
		} else {
			const offset = range.min * this.stride;
			const length = (range.max - range.min + 1) * this.stride;
			this.glBuffer.subData(offset, this.localBuffer, offset, length);
		}
		this.glBuffer.bindNone();
	}

	bind() {
		if (this.dirty) {
			this.flush();
		}
		this.glBuffer.bind();
	}

	bindNone() {
		this.glBuffer.bindNone();
	}

	protected disposeImpl(): void {
		this.glBuffer.dispose();
	}
}