export class Vector3 {
	constructor(
		readonly x: number,
		readonly y: number,
		readonly z: number,
	) { }

	get negated(): Vector3 {
		return new Vector3(
			-this.x,
			-this.y,
			-this.z,
		);
	}

	add(other: Vector3): Vector3 {
		return new Vector3(
			this.x + other.x,
			this.y + other.y,
			this.z + other.z,
		);
	}

	sub(other: Vector3): Vector3 {
		return new Vector3(
			this.x - other.x,
			this.y - other.y,
			this.z - other.z,
		);
	}

	mul(other: number): Vector3 {
		return new Vector3(
			this.x * other,
			this.y * other,
			this.z * other,
		);
	}

	div(other: number): Vector3 {
		return new Vector3(
			this.x / other,
			this.y / other,
			this.z / other,
		);
	}

	get magnitudeSquared(): number {
		return this.x * this.x + this.y * this.y + this.z * this.z;
	}

	get magnitude(): number {
		return Math.sqrt(this.magnitudeSquared);
	}

	get normalized(): Vector3 {
		return this.div(this.magnitude);
	}

	dot(other: Vector3): number {
		return this.x * other.x + this.y * other.y + this.z * other.z;
	}

	cross(other: Vector3): Vector3 {
		return new Vector3(
			this.y * other.z - this.z * other.y,
			this.z * other.x - this.x * other.z,
			this.x * other.y - this.y * other.x,
		);
	}
}