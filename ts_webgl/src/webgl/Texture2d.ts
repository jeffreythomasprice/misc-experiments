import { Size2 } from "../geometry";
import { Disposable, loadImageFromURL } from "../utils";

export class Texture2d extends Disposable {
	private readonly texture: WebGLTexture;
	private size_: Size2;

	static async createFromURL(gl: WebGL2RenderingContext, url: URL): Promise<Texture2d> {
		const image = await loadImageFromURL(url);
		const result = new Texture2d(gl);
		result.texImage(0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
		return result;
	}

	constructor(
		private readonly gl: WebGL2RenderingContext,
	) {
		super();

		const texture = gl.createTexture();
		if (!texture) {
			throw new Error("error creating texture");
		}
		this.texture = texture;
		this.size_ = new Size2(0, 0);
	}

	get size() {
		return this.size_;
	}

	get width() {
		return this.size_.width;
	}

	get height() {
		return this.size_.height;
	}

	bind() {
		this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
	}

	bindNone() {
		this.gl.bindTexture(this.gl.TEXTURE_2D, null);
	}

	// TODO should use enums
	texImage(level: GLint, internalformat: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, type: GLenum, pixels: ArrayBufferView | null): void;
	texImage(level: GLint, internalformat: GLint, format: GLenum, type: GLenum, source: TexImageSource): void;
	texImage(level: GLint, internalformat: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, type: GLenum, pboOffset: GLintptr): void;
	texImage(level: GLint, internalformat: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, type: GLenum, source: TexImageSource): void;
	texImage(level: GLint, internalformat: GLint, width: GLsizei, height: GLsizei, border: GLint, format: GLenum, type: GLenum, srcData: ArrayBufferView, srcOffset: GLuint): void;
	texImage(level: GLint, internalformat: GLint, widthOrFormat: GLsizei | GLenum, heightOrType: GLsizei | GLenum, borderOrSource: GLint | TexImageSource, format?: GLenum, type?: GLenum, data?: ArrayBufferView | GLintptr | TexImageSource | ArrayBufferView | null, srcOffset?: GLuint) {
		this.bind();
		if (isTexImageSource(borderOrSource)) {
			this.gl.texImage2D(this.gl.TEXTURE_2D, level, internalformat, widthOrFormat, heightOrType, borderOrSource);
			this.size_ = new Size2(borderOrSource.width, borderOrSource.height);
		} else if (typeof srcOffset === "number") {
			this.gl.texImage2D(this.gl.TEXTURE_2D, level, internalformat, widthOrFormat, heightOrType, borderOrSource, format as number, type as number, data as ArrayBufferView, srcOffset);
			this.size_ = new Size2(widthOrFormat, heightOrType);
		} else if (isTexImageSource(data)) {
			this.gl.texImage2D(this.gl.TEXTURE_2D, level, internalformat, widthOrFormat, heightOrType, borderOrSource, format as number, type as number, data);
			this.size_ = new Size2(widthOrFormat, heightOrType);
		} else if (isArrayBufferView(data) || data === null) {
			this.gl.texImage2D(this.gl.TEXTURE_2D, level, internalformat, widthOrFormat, heightOrType, borderOrSource, format as number, type as number, data);
			this.size_ = new Size2(widthOrFormat, heightOrType);
		} else if (data !== undefined) {
			this.gl.texImage2D(this.gl.TEXTURE_2D, level, internalformat, widthOrFormat, heightOrType, borderOrSource, format as number, type as number, data);
			this.size_ = new Size2(widthOrFormat, heightOrType);
		} else {
			throw new Error("unreachable");
		}
		// TODO different params for power of 2 sizes
		this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.LINEAR);
		this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST);
		this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
		this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
		this.bindNone();
	}

	protected disposeImpl(): void {
		this.gl.deleteTexture(this.texture);
	}
}

function isTexImageSource(x: unknown): x is TexImageSource {
	return typeof x === "object" && (
		x instanceof ImageBitmap ||
		x instanceof ImageData ||
		x instanceof HTMLImageElement ||
		x instanceof HTMLCanvasElement ||
		x instanceof HTMLVideoElement ||
		x instanceof OffscreenCanvas
	);
}

function isArrayBufferView(x: unknown): x is ArrayBufferView {
	return typeof x === "object" &&
		(x as ArrayBufferView).buffer instanceof ArrayBuffer &&
		typeof (x as ArrayBufferView).byteLength === "number" &&
		typeof (x as ArrayBufferView).byteOffset === "number";
}