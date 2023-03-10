import { Size2 } from "./Size2";
import { Vector2 } from "./Vector2";

/**
 * Axially-aligned bounding-box, 2d
 */
export class Aabb2 {
	static fromPoints(points: Vector2[]): Aabb2 {
		if (points.length < 1) {
			throw new Error("must provide at least one point");
		}
		let x1 = points[0].x;
		let y1 = points[0].y;
		let x2 = x1;
		let y2 = y1;
		for (const p of points) {
			x1 = Math.min(x1, p.x);
			y1 = Math.min(y1, p.y);
			x2 = Math.max(x2, p.x);
			y2 = Math.max(y2, p.y);
		}
		return new Aabb2(new Vector2(x1, y1), new Vector2(x2, y2));
	}

	readonly min: Vector2;
	readonly max: Vector2;

	constructor(min: Vector2, max: Vector2);
	constructor(origin: Vector2, size: Size2);
	constructor(min: Vector2, maxOrSize: Vector2 | Size2) {
		this.min = min;
		if (maxOrSize instanceof Vector2) {
			this.max = maxOrSize;
		} else {
			this.max = this.min.add(maxOrSize.toVector);
		}
	}

	toString(): string {
		return `AABB(min=${this.min}, max=${this.max})`;
	}

	get origin(): Vector2 {
		return this.min;
	}

	get x(): number {
		return this.min.x;
	}

	get y(): number {
		return this.min.y;
	}

	get size(): Size2 {
		return this.max.sub(this.min).toSize;
	}

	get width(): number {
		return this.size.width;
	}

	get height(): number {
		return this.size.height;
	}
}