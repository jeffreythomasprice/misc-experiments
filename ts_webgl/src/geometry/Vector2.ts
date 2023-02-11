export class Vector2 {
	constructor(
		readonly x: number,
		readonly y: number,
	) { }

	get negated(): Vector2 {
		return new Vector2(
			-this.x,
			-this.y,
		);
	}

	add(other: Vector2): Vector2 {
		return new Vector2(
			this.x + other.x,
			this.y + other.y,
		);
	}

	sub(other: Vector2): Vector2 {
		return new Vector2(
			this.x - other.x,
			this.y - other.y,
		);
	}

	mul(other: number): Vector2 {
		return new Vector2(
			this.x * other,
			this.y * other,
		);
	}

	div(other: number): Vector2 {
		return new Vector2(
			this.x / other,
			this.y / other,
		);
	}

	get magnitudeSquared(): number {
		return this.x * this.x + this.y * this.y;
	}

	get magnitude(): number {
		return Math.sqrt(this.magnitudeSquared);
	}

	get normalized(): Vector2 {
		return this.div(this.magnitude);
	}

	dot(other: Vector2): number {
		return this.x * other.x + this.y * other.y;
	}
}