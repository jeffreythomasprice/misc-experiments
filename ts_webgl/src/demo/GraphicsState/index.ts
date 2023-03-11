import texturedShaderVertexSource from "bundle-text:./texturedShader.vertex";
import texturedShaderFragmentSource from "bundle-text:./texturedShader.fragment";
import untexturedShaderVertexSource from "bundle-text:./untexturedShader.vertex";
import untexturedShaderFragmentSource from "bundle-text:./untexturedShader.fragment";

import { Mesh, Shader, StructIO, Texture2d, VertexDefinition } from "../../webgl";
import { Matrix4, Rgba, Size2, Vector2 } from "../../geometry";
import { Disposable } from "../../utils";

class VertexPos2CoordRgbaWriter extends StructIO<GraphicsState.VertexPos2CoordRgba> {
	read(source: Buffer, offset: number): GraphicsState.VertexPos2CoordRgba {
		const x = source.readFloatLE(offset);
		offset += 4;
		const y = source.readFloatLE(offset);
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
		return new GraphicsState.VertexPos2CoordRgba(
			new Vector2(x, y),
			new Vector2(tx, ty),
			new Rgba(r, g, b, a),
		);
	}

	write(destination: Buffer, offset: number, source: GraphicsState.VertexPos2CoordRgba): void {
		offset = destination.writeFloatLE(source.position.x, offset);
		offset = destination.writeFloatLE(source.position.y, offset);
		offset = destination.writeFloatLE(source.textureCoordinate.x, offset);
		offset = destination.writeFloatLE(source.textureCoordinate.y, offset);
		offset = destination.writeFloatLE(source.color.red, offset);
		offset = destination.writeFloatLE(source.color.green, offset);
		offset = destination.writeFloatLE(source.color.blue, offset);
		offset = destination.writeFloatLE(source.color.alpha, offset);
	}
}

class VertexPos2RgbaWriter extends StructIO<GraphicsState.VertexPos2Rgba> {
	read(source: Buffer, offset: number): GraphicsState.VertexPos2Rgba {
		const x = source.readFloatLE(offset);
		offset += 4;
		const y = source.readFloatLE(offset);
		offset += 4;
		const r = source.readFloatLE(offset);
		offset += 4;
		const g = source.readFloatLE(offset);
		offset += 4;
		const b = source.readFloatLE(offset);
		offset += 4;
		const a = source.readFloatLE(offset);
		offset += 4;
		return new GraphicsState.VertexPos2Rgba(
			new Vector2(x, y),
			new Rgba(r, g, b, a),
		);
	}

	write(destination: Buffer, offset: number, source: GraphicsState.VertexPos2Rgba): void {
		offset = destination.writeFloatLE(source.position.x, offset);
		offset = destination.writeFloatLE(source.position.y, offset);
		offset = destination.writeFloatLE(source.color.red, offset);
		offset = destination.writeFloatLE(source.color.green, offset);
		offset = destination.writeFloatLE(source.color.blue, offset);
		offset = destination.writeFloatLE(source.color.alpha, offset);
	}
}

export class GraphicsState extends Disposable {

	private texturedShader: Shader;
	private texturedShaderSamplerUniform: Shader.UniformInfo;
	private texturedShaderProjectionMatrixUniform: Shader.UniformInfo;
	private texturedShaderModelviewMatrixUniform: Shader.UniformInfo;

	private untexturedShader: Shader;
	private untexturedShaderProjectionMatrixUniform: Shader.UniformInfo;
	private untexturedShaderModelviewMatrixUniform: Shader.UniformInfo;

	private texturedVertexWriter: VertexPos2CoordRgbaWriter;
	private untexturedVertexWriter: VertexPos2RgbaWriter;

	private _texture: Texture2d | null = null;
	private _transparency = false;
	private _projectionSize: Size2 = new Size2(0, 0);
	private _modelviewMatrix: Matrix4 = Matrix4.identity;
	private _isDrawing = false;
	private _activeShader: Shader | null = null;

	constructor(
		private readonly gl: WebGL2RenderingContext,
	) {
		super();

		this.texturedShader = new Shader(gl, texturedShaderVertexSource, texturedShaderFragmentSource);
		this.texturedShaderSamplerUniform = this.texturedShader.assertUniform("samplerUniform");
		this.texturedShaderProjectionMatrixUniform = this.texturedShader.assertUniform("projectionMatrixUniform");
		this.texturedShaderModelviewMatrixUniform = this.texturedShader.assertUniform("modelviewMatrixUniform");

		this.untexturedShader = new Shader(gl, untexturedShaderVertexSource, untexturedShaderFragmentSource);
		this.untexturedShaderProjectionMatrixUniform = this.untexturedShader.assertUniform("projectionMatrixUniform");
		this.untexturedShaderModelviewMatrixUniform = this.untexturedShader.assertUniform("modelviewMatrixUniform");

		this.texturedVertexWriter = new VertexPos2CoordRgbaWriter(new VertexDefinition.Builder(this.gl, this.texturedShader)
			.attribute("positionAttribute", (builder) => {
				builder
					.size(2)
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
			.build());

		this.untexturedVertexWriter = new VertexPos2RgbaWriter(new VertexDefinition.Builder(this.gl, this.untexturedShader)
			.attribute("positionAttribute", (builder) => {
				builder
					.size(2)
					.type(WebGL2RenderingContext.FLOAT);
			})
			.attribute("colorAttribute", (builder) => {
				builder
					.size(4)
					.type(WebGL2RenderingContext.FLOAT);
			})
			.build());
	}

	createTexturedMesh(): GraphicsState.TexturedMesh {
		return new Mesh(this.gl, this.texturedVertexWriter);
	}

	createUntexturedMesh(): GraphicsState.UntexturedMesh {
		return new Mesh(this.gl, this.untexturedVertexWriter);
	}

	get texture(): Texture2d | null {
		return this._texture;
	}

	set texture(t: Texture2d | null) {
		this._texture = t;
		if (t && this._activeShader === this.texturedShader) {
			t.bind();
		}
	}

	get transparency(): boolean {
		return this._transparency;
	}

	set transparency(b: boolean) {
		const update = this._transparency !== b && this._isDrawing;
		this._transparency = b;
		if (update) {
			this.setStateTransparency();
		}
	}

	get projectionSize(): Size2 {
		return this._projectionSize;
	}

	set projectionSize(s: Size2) {
		this._projectionSize = s;
		if (this._activeShader === this.texturedShader) {
			this.gl.uniformMatrix4fv(
				this.texturedShaderProjectionMatrixUniform.location,
				false,
				this.projectionMatrix.toArray()
			);
		} else if (this._activeShader === this.untexturedShader) {
			this.gl.uniformMatrix4fv(
				this.untexturedShaderProjectionMatrixUniform.location,
				false,
				this.projectionMatrix.toArray()
			);
		}
	}

	get projectionMatrix(): Matrix4 {
		return Matrix4.createOrtho(0, this._projectionSize.width, this._projectionSize.height, 0, -1, 1);
	}

	get modelviewMatrix(): Matrix4 {
		return this._modelviewMatrix;
	}

	set modelviewMatrix(m: Matrix4) {
		this._modelviewMatrix = m;
		if (this._activeShader === this.texturedShader) {
			this.gl.uniformMatrix4fv(
				this.texturedShaderModelviewMatrixUniform.location,
				false,
				this.modelviewMatrix.toArray()
			);
		} else if (this._activeShader === this.untexturedShader) {
			this.gl.uniformMatrix4fv(
				this.untexturedShaderModelviewMatrixUniform.location,
				false,
				this.modelviewMatrix.toArray()
			);
		}
	}

	begin() {
		if (this._isDrawing) {
			throw new Error("already drawing");
		}
		this._isDrawing = true;
	}

	render(mesh: GraphicsState.TexturedMesh | GraphicsState.UntexturedMesh, first: number, count: number) {
		if (mesh.writer === this.texturedVertexWriter) {
			if (this._activeShader !== this.texturedShader) {
				this.setStateToTextured();
			}
		} else if (mesh.writer === this.untexturedVertexWriter) {
			if (this._activeShader !== this.untexturedShader) {
				this.setStateToUntextured();
			}
		} else {
			throw new Error("unhandled mesh type");
		}
		mesh.bind();
		this.gl.drawElements(this.gl.TRIANGLES, count, this.gl.UNSIGNED_SHORT, first * 2);
		mesh.bindNone();
	}

	end() {
		if (!this._isDrawing) {
			throw new Error("not drawing");
		}
		this._isDrawing = false;
		this.setStateToNotDrawing();
	}

	protected disposeImpl(): void {
		this.texturedShader.dispose();
		this.untexturedShader.dispose();
	}

	private setStateToNotDrawing() {
		if (this._activeShader) {
			this._activeShader.useNone();
			this._activeShader = null;
		}

		if (this._texture) {
			this._texture.bindNone();
		}

		this.gl.disable(this.gl.BLEND);
	}

	private setStateToTextured() {
		if (this._activeShader && this._activeShader !== this.texturedShader) {
			this.setStateToNotDrawing();
		}

		this._activeShader = this.texturedShader;
		this._activeShader.use();

		this.gl.activeTexture(this.gl.TEXTURE0);
		this.gl.uniform1i(this.texturedShaderSamplerUniform.location, 0);
		if (this._texture) {
			this._texture.bind();
		}

		this.setStateTransparency();

		this.gl.uniformMatrix4fv(
			this.texturedShaderProjectionMatrixUniform.location,
			false,
			this.projectionMatrix.toArray()
		);
		this.gl.uniformMatrix4fv(
			this.texturedShaderModelviewMatrixUniform.location,
			false,
			this.modelviewMatrix.toArray()
		);
	}

	private setStateToUntextured() {
		if (this._activeShader && this._activeShader !== this.untexturedShader) {
			this.setStateToNotDrawing();
		}

		this._activeShader = this.untexturedShader;
		this._activeShader.use();

		this.setStateTransparency();

		this.gl.uniformMatrix4fv(
			this.untexturedShaderProjectionMatrixUniform.location,
			false,
			this.projectionMatrix.toArray()
		);
		this.gl.uniformMatrix4fv(
			this.untexturedShaderModelviewMatrixUniform.location,
			false,
			this.modelviewMatrix.toArray()
		);
	}

	private setStateTransparency() {
		if (this._transparency) {
			this.gl.blendFunc(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA);
			this.gl.enable(this.gl.BLEND);
		} else {
			this.gl.disable(this.gl.BLEND);
		}
	}
}

export namespace GraphicsState {
	export class VertexPos2CoordRgba {
		constructor(
			readonly position: Vector2,
			readonly textureCoordinate: Vector2,
			readonly color: Rgba,
		) { }
	}

	export class VertexPos2Rgba {
		constructor(
			readonly position: Vector2,
			readonly color: Rgba,
		) { }
	}

	export type TexturedMesh = Mesh<GraphicsState.VertexPos2CoordRgba>;

	export type UntexturedMesh = Mesh<GraphicsState.VertexPos2Rgba>;
}
