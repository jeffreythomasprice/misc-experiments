import { Disposable } from "../utils";

export class Shader extends Disposable {
	private readonly program: WebGLProgram;

	readonly attributes: Map<string, Shader.AttributeInfo>;
	readonly uniforms: Map<string, Shader.UniformInfo>;

	constructor(
		private readonly gl: WebGL2RenderingContext,
		vertexSource: string,
		fragmentSource: string,
	) {
		super();

		const vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexSource);

		let fragmentShader;
		try {
			fragmentShader = createShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
		} catch (e) {
			gl.deleteShader(vertexShader);
			throw e;
		}

		const program = gl.createProgram();
		if (!program) {
			gl.deleteShader(vertexShader);
			gl.deleteShader(fragmentShader);
			throw new Error("error creating shader program");
		}

		gl.attachShader(program, vertexShader);
		gl.attachShader(program, fragmentShader);
		gl.linkProgram(program);
		gl.deleteShader(vertexShader);
		gl.deleteShader(fragmentShader);
		if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
			const log = gl.getProgramInfoLog(program);
			gl.deleteProgram(program);
			throw new Error(`error linking shader program\n${log}`);
		}

		const attributes = new Map<string, Shader.AttributeInfo>();
		for (let i = 0; i < gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES); i++) {
			const info = gl.getActiveAttrib(program, i);
			if (!info) {
				throw new Error(`expected an attribute at index ${i}`);
			}
			const location = gl.getAttribLocation(program, info.name);
			if (location < 0) {
				throw new Error(`failed to get attribute location ${info.name}`);
			}
			attributes.set(info.name, {
				name: info.name,
				location
			});
		}

		const uniforms = new Map<string, Shader.UniformInfo>();
		for (let i = 0; i < gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS); i++) {
			const info = gl.getActiveUniform(program, i);
			if (!info) {
				throw new Error(`expected a uniform at index ${i}`);
			}
			const location = gl.getUniformLocation(program, info.name);
			if (!location) {
				throw new Error(`failed to get uniform location ${info.name}`);
			}
			uniforms.set(info.name, {
				name: info.name,
				location
			});
		}

		this.program = program;
		this.attributes = attributes;
		this.uniforms = uniforms;
	}

	use() {
		this.gl.useProgram(this.program);
	}

	useNone() {
		this.gl.useProgram(null);
	}

	protected disposeImpl(): void {
		this.gl.deleteProgram(this.program);
	}
}

export namespace Shader {
	export interface AttributeInfo {
		name: string;
		location: number;
	}

	export interface UniformInfo {
		name: string;
		location: WebGLUniformLocation;
	}
}

function createShader(gl: WebGL2RenderingContext, type: number, source: string) {
	const result = gl.createShader(type);
	if (!result) {
		throw new Error(`error creating shader of type ${getShaderTypeString(gl, type)}`);
	}
	gl.shaderSource(result, source);
	gl.compileShader(result);
	if (!gl.getShaderParameter(result, gl.COMPILE_STATUS)) {
		const log = gl.getShaderInfoLog(result);
		gl.deleteShader(result);
		throw new Error(`error compiling shader of type ${getShaderTypeString(gl, type)}\n${log}`);
	}
	return result;
}

function getShaderTypeString(gl: WebGL2RenderingContext, type: number) {
	switch (type) {
		case gl.VERTEX_SHADER:
			return "VERTEX_SHADER";
		case gl.FRAGMENT_SHADER:
			return "FRAGMENT_SHADER";
		default:
			return `${type}`;
	}
}