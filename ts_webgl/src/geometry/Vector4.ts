import { Rgba } from "./Rgba";

export class Vector4 {
	constructor(
		readonly x: number,
		readonly y: number,
		readonly z: number,
		readonly w: number,
	) { }

	toString(): string {
		return `(${this.x}, ${this.y}, ${this.z}, ${this.w})`;
	}

	get toRgba(): Rgba {
		return new Rgba(this.x, this.y, this.z, this.w);
	}

	get negated(): Vector4 {
		return new Vector4(
			-this.x,
			-this.y,
			-this.z,
			-this.w,
		);
	}

	add(other: Vector4): Vector4 {
		return new Vector4(
			this.x + other.x,
			this.y + other.y,
			this.z + other.z,
			this.w + other.w,
		);
	}

	sub(other: Vector4): Vector4 {
		return new Vector4(
			this.x - other.x,
			this.y - other.y,
			this.z - other.z,
			this.w - other.w,
		);
	}

	mul(other: number): Vector4 {
		return new Vector4(
			this.x * other,
			this.y * other,
			this.z * other,
			this.w * other,
		);
	}

	div(other: number): Vector4 {
		return new Vector4(
			this.x / other,
			this.y / other,
			this.z / other,
			this.w / other,
		);
	}

	get magnitudeSquared(): number {
		return this.x * this.x + this.y * this.y + this.z * this.z + this.w * this.w;
	}

	get magnitude(): number {
		return Math.sqrt(this.magnitudeSquared);
	}

	get normalized(): Vector4 {
		return this.div(this.magnitude);
	}

	dot(other: Vector4): number {
		return this.x * other.x + this.y * other.y + this.z * other.z + this.w * other.w;
	}
}