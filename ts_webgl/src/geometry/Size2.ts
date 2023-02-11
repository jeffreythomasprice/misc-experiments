import { Vector2 } from "./Vector2";

export class Size2 extends Vector2 {
	constructor(
		width: number,
		height: number,
	) {
		super(width, height);
	}

	get width() {
		return this.x;
	}

	get height() {
		return this.y;
	}
}