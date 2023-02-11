import { Vector2 } from "./Vector2";

export class Size2 {
	constructor(
		readonly width: number,
		readonly height: number,
	) { }

	toString(): string {
		return `(${this.width} x ${this.height})`;
	}

	get toVector(): Vector2 {
		return new Vector2(this.width, this.height);
	}
}