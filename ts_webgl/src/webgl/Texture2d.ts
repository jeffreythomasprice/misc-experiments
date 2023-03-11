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

	texImage(level: GLint, internalformat: Texture2d.Format, width: GLsizei, height: GLsizei, border: GLint, format: Texture2d.Format, type: Texture2d.Type, pixels: ArrayBufferView | null): void;
	texImage(level: GLint, internalformat: Texture2d.Format, format: Texture2d.Format, type: Texture2d.Type, source: TexImageSource): void;
	texImage(level: GLint, internalformat: Texture2d.Format, width: GLsizei, height: GLsizei, border: GLint, format: Texture2d.Format, type: Texture2d.Type, pboOffset: GLintptr): void;
	texImage(level: GLint, internalformat: Texture2d.Format, width: GLsizei, height: GLsizei, border: GLint, format: Texture2d.Format, type: Texture2d.Type, source: TexImageSource): void;
	texImage(level: GLint, internalformat: Texture2d.Format, width: GLsizei, height: GLsizei, border: GLint, format: Texture2d.Format, type: Texture2d.Type, srcData: ArrayBufferView, srcOffset: GLuint): void;
	texImage(level: GLint, internalformat: Texture2d.Format, widthOrFormat: GLsizei | GLenum, heightOrType: GLsizei | GLenum, borderOrSource: GLint | TexImageSource, format?: GLenum, type?: GLenum, data?: ArrayBufferView | GLintptr | TexImageSource | ArrayBufferView | null, srcOffset?: GLuint) {
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
		if (isPowerOf2(this.width) && isPowerOf2(this.height)) {
			this.gl.generateMipmap(this.gl.TEXTURE_2D);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.LINEAR);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST_MIPMAP_LINEAR);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.REPEAT);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.REPEAT);
		} else {
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.LINEAR);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
			this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
		}
		this.bindNone();
	}

	protected disposeImpl(): void {
		this.gl.deleteTexture(this.texture);
	}
}

export namespace Texture2d {
	export enum Format {
		RGBA = 0x1908,
		RGB = 0x1907,
		LUMINANCE_ALPHA = 0x190A,
		LUMINANCE = 0x1909,
		ALPHA = 0x1906,
		R8 = 0x8229,
		R8_SNORM = 0x8F94,
		RG8 = 0x822B,
		RG8_SNORM = 0x8F95,
		RGB8 = 0x8051,
		RGB8_SNORM = 0x8F96,
		RGB565 = 0x8D62,
		RGBA4 = 0x8056,
		RGB5_A1 = 0x8057,
		RGBA8 = 0x8058,
		RGBA8_SNORM = 0x8F97,
		RGB10_A2 = 0x8059,
		RGB10_A2UI = 0x906F,
		SRGB8 = 0x8C41,
		SRGB8_ALPHA8 = 0x8C43,
		R16F = 0x822D,
		RG16F = 0x822F,
		RGB16F = 0x881B,
		RGBA16F = 0x881A,
		R32F = 0x822E,
		RG32F = 0x8230,
		RGB32F = 0x8815,
		RGBA32F = 0x8814,
		R11F_G11F_B10F = 0x8C3A,
		RGB9_E5 = 0x8C3D,
		R8I = 0x8231,
		R8UI = 0x8232,
		R16I = 0x8233,
		R16UI = 0x8234,
		R32I = 0x8235,
		R32UI = 0x8236,
		RG8I = 0x8237,
		RG8UI = 0x8238,
		RG16I = 0x8239,
		RG16UI = 0x823A,
		RG32I = 0x8235,
		RG32UI = 0x8236,
		RGB8I = 0x8D8F,
		RGB8UI = 0x8D7D,
		RGB16I = 0x8D89,
		RGB16UI = 0x8D77,
		RGB32I = 0x8D83,
		RGB32UI = 0x8D71,
		RGBA8I = 0x8D8E,
		RGBA8UI = 0x8D7C,
		RGBA16I = 0x8D88,
		RGBA16UI = 0x8D76,
		RGBA32I = 0x8D82,
		RGBA32UI = 0x8D70,
	}

	export enum Type {
		UNSIGNED_BYTE = 0x1401,
		UNSIGNED_SHORT_5_6_5 = 0x8363,
		UNSIGNED_SHORT_4_4_4_4 = 0x8033,
		UNSIGNED_SHORT_5_5_5_1 = 0x8034,
		UNSIGNED_SHORT = 0x1403,
		UNSIGNED_INT = 0x1405,
		FLOAT = 0x1406,
		BYTE = 0x1400,
		SHORT = 0x1402,
		INT = 0x1404,
		HALF_FLOAT = 0x140B,
		UNSIGNED_INT_2_10_10_10_REV = 0x8368,
		UNSIGNED_INT_10F_11F_11F_REV = 0x8C3B,
		UNSIGNED_INT_5_9_9_9_REV = 0x8C3E,
		UNSIGNED_INT_24_8 = 0x84FA,
		FLOAT_32_UNSIGNED_INT_24_8_REV = 0x8DAD,
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

function isPowerOf2(x: number): boolean {
	return (x != 0) && !(x & (x - 1));
}