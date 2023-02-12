import { Shader } from "./Shader";

export interface VertexDefinition {
	enable(): void;
	disable(): void;
}

export namespace VertexDefinition {
	export enum DataType {
		Byte = 0x1400,
		UnsignedByte = 0x1401,
		Short = 0x1402,
		UnsignedShort = 0x1403,
		Int = 0x1404,
		UnsignedInt = 0x1405,
		Int_2_10_10_10 = 0x8D9F,
		UnsignedInt_2_10_10_10 = 0x8368,
		HalfFloat = 0x140B,
		Float = 0x1406,
	}

	export interface AttributeBuilder {
		size(value: number): AttributeBuilder;
		type(value: DataType): AttributeBuilder;
		normalized(value: boolean): AttributeBuilder;
		offset(value: number): AttributeBuilder;
	}

	export class Builder {
		private readonly attributes = new Map<string, Attribute>();
		private stride_ = 0;

		constructor(
			private readonly gl: WebGL2RenderingContext,
			private readonly shader: Shader,
		) { }

		attribute(name: string, f: (attrBuilder: AttributeBuilder) => void): Builder {
			if (this.attributes.has(name)) {
				throw new Error(`duplicate definition for ${name}`);
			}
			const info = this.shader.attributes.get(name);
			if (!info) {
				throw new Error(`no such attribute ${name}`);
			}
			const attrBuilder = new AttributeBuilderImpl(info);
			// guess where the offset for this one should start based on the previous attributes
			attrBuilder.offset(this.calculateStride());
			f(attrBuilder);
			this.attributes.set(name, attrBuilder.build());
			return this;
		}

		stride(value: number): Builder {
			this.stride_ = value;
			return this;
		}

		build(): VertexDefinition {
			if (this.attributes.size !== this.shader.attributes.size) {
				throw new Error(`expected ${this.shader.attributes.size} atribute definitions, got ${this.attributes.size}`);
			}
			return new VertexDefinitionImpl(
				this.gl,
				this.stride_ || this.calculateStride(),
				...this.attributes.values()
			);
		}

		private calculateStride(): number {
			let result = 0;
			for (const attr of this.attributes.values()) {
				result = Math.max(result, attr.offset + attr.byteLength);
			}
			return result;
		}
	}
}

class AttributeBuilderImpl implements VertexDefinition.AttributeBuilder {
	private size_: number | undefined;
	private type_: VertexDefinition.DataType | undefined;
	private normalized_ = false;
	private offset_ = 0;

	constructor(private readonly info: Shader.AttributeInfo) { }

	size(value: number): VertexDefinition.AttributeBuilder {
		this.size_ = value;
		return this;
	}

	type(value: VertexDefinition.DataType): VertexDefinition.AttributeBuilder {
		this.type_ = value;

		return this;
	}

	normalized(value: boolean): VertexDefinition.AttributeBuilder {
		this.normalized_ = value;
		return this;
	}

	offset(value: number): VertexDefinition.AttributeBuilder {
		this.offset_ = value;
		return this;
	}

	build(): Attribute {
		if (this.size_ === undefined) {
			throw new Error("must provide a size");
		}
		if (this.type_ === undefined) {
			throw new Error("must provide a type");
		}
		return new Attribute(this.info.location, this.size_, this.type_, this.normalized_, this.offset_);
	}
}

class Attribute {
	readonly bytesPerComponent: number;
	readonly byteLength: number;

	constructor(
		readonly location: number,
		readonly size: number,
		readonly type: VertexDefinition.DataType,
		readonly normalized: boolean,
		readonly offset: number
	) {
		this.bytesPerComponent = getBytesPerComponent(type);
		this.byteLength = this.bytesPerComponent * this.size;
	}
}

class VertexDefinitionImpl implements VertexDefinition {
	private attributes: ReadonlyArray<Attribute>;

	constructor(
		private readonly gl: WebGL2RenderingContext,
		private readonly stride: number,
		...attributes: Attribute[]
	) {
		this.attributes = [...attributes];
	}

	enable(): void {
		for (const attr of this.attributes) {
			this.gl.enableVertexAttribArray(attr.location);
			this.gl.vertexAttribPointer(
				attr.location,
				attr.size,
				attr.type,
				attr.normalized,
				this.stride,
				attr.offset,
			);
		}
	}

	disable(): void {
		for (const attr of this.attributes) {
			this.gl.disableVertexAttribArray(attr.location);
		}
	}
}

function getBytesPerComponent(type: VertexDefinition.DataType) {
	switch (type) {
		case VertexDefinition.DataType.Byte:
		case VertexDefinition.DataType.UnsignedByte:
			return 1;
		case VertexDefinition.DataType.Short:
		case VertexDefinition.DataType.UnsignedShort:
		case VertexDefinition.DataType.HalfFloat:
			return 2;
		case VertexDefinition.DataType.Float:
		case VertexDefinition.DataType.Int:
		case VertexDefinition.DataType.UnsignedInt:
		case VertexDefinition.DataType.Int_2_10_10_10:
		case VertexDefinition.DataType.UnsignedInt_2_10_10_10:
			return 4;
		default:
			throw new Error(`unrecognized type ${type}`);
	}
}