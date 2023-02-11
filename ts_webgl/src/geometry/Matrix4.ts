import { Vector3 } from "./Vector3";

export class Matrix4 {
	constructor(
		readonly d00: number, readonly d01: number, readonly d02: number, readonly d03: number,
		readonly d10: number, readonly d11: number, readonly d12: number, readonly d13: number,
		readonly d20: number, readonly d21: number, readonly d22: number, readonly d23: number,
		readonly d30: number, readonly d31: number, readonly d32: number, readonly d33: number,
	) { }

	static readonly identity = new Matrix4(
		1, 0, 0, 0,
		0, 1, 0, 0,
		0, 0, 1, 0,
		0, 0, 0, 1,
	);

	static translation(v: Vector3): Matrix4 {
		return new Matrix4(
			1, 0, 0, 0,
			0, 1, 0, 0,
			0, 0, 1, 0,
			v.x, v.y, v.z, 1,
		);
	}

	static scale(v: Vector3): Matrix4 {
		return new Matrix4(
			v.x, 0, 0, 0,
			0, v.y, 0, 0,
			0, 0, v.z, 0,
			0, 0, 0, 1,
		);
	}

	/**
	 * @param axis in radians
	 */
	static rotation(v: Vector3, axis: number): Matrix4 {
		throw new Error("TODO");
	}

	static ortho(left: number, right: number, bottom: number, top: number, near: number, far: number): Matrix4 {
		return new Matrix4(
			2 / (right - left), 0, 0, - (right + left) / (right - left),
			0, 2 / (top - bottom), 0, - (top + bottom) / (top - bottom),
			0, 0, -2 / (far - near), -(far + near) / (far - near),
			0, 0, 0, 1,
		);
	}

	toArray(): number[] {
		return [
			this.d00, this.d10, this.d20, this.d30,
			this.d01, this.d11, this.d21, this.d31,
			this.d02, this.d12, this.d22, this.d32,
			this.d03, this.d13, this.d23, this.d33,
		];
	}

	mul(other: Matrix4) {
		throw new Error("TODO");
	}
}