import { VertexDefinition } from "./VertexDefinition";

export abstract class StructIO<T> {
	constructor(readonly def: VertexDefinition) { }

	abstract read(source: Buffer, offset: number): T;
	abstract write(destination: Buffer, offset: number, source: T): void;

	create(...source: T[]): Buffer {
		const result = Buffer.allocUnsafe(this.def.stride * source.length);
		this.update(result, 0, ...source);
		return result;
	}

	update(destination: Buffer, offset: number, ...source: T[]) {
		for (const x of source) {
			this.write(destination, offset, x);
			offset += this.def.stride;
		}
	}
}