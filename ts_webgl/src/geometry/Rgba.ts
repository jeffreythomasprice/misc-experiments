import { Vector4 } from "./Vector4";

export class Rgba {
	constructor(
		readonly red: number,
		readonly green: number,
		readonly blue: number,
		readonly alpha: number,
	) { }

	toString(): string {
		return `RGBA(${this.red}, ${this.green}, ${this.blue}, ${this.alpha})`;
	}

	get toVector(): Vector4 {
		return new Vector4(this.red, this.green, this.blue, this.alpha);
	}
}