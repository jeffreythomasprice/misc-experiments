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

	static createTranslation(v: Vector3): Matrix4 {
		return new Matrix4(
			1, 0, 0, 0,
			0, 1, 0, 0,
			0, 0, 1, 0,
			v.x, v.y, v.z, 1,
		);
	}

	static createScale(v: Vector3): Matrix4 {
		return new Matrix4(
			v.x, 0, 0, 0,
			0, v.y, 0, 0,
			0, 0, v.z, 0,
			0, 0, 0, 1,
		);
	}

	/**
	 * @param angle in radians
	 */
	static createRotation(axis: Vector3, angle: number): Matrix4 {
		const c = Math.cos(angle);
		const s = Math.sin(angle);
		axis = axis.normalized;
		return new Matrix4(
			axis.x * axis.x * (1 - c) + c, axis.x * axis.y * (1 - c) - axis.z * s, axis.x * axis.z * (1 - c) + axis.y * s, 0,
			axis.y * axis.x * (1 - c) + axis.z * s, axis.y * axis.y * (1 - c) + c, axis.y * axis.z * (1 - c) - axis.x * s, 0,
			axis.x * axis.z * (1 - c) - axis.y * s, axis.y * axis.z * (1 - c) + axis.x * s, axis.z * axis.z * (1 - c) + c, 0,
			0, 0, 0, 1,
		);
	}

	static createOrtho(left: number, right: number, bottom: number, top: number, near: number, far: number): Matrix4 {
		return new Matrix4(
			2 / (right - left), 0, 0, - (right + left) / (right - left),
			0, 2 / (top - bottom), 0, - (top + bottom) / (top - bottom),
			0, 0, -2 / (far - near), -(far + near) / (far - near),
			0, 0, 0, 1,
		);
	}

	/**
	 * @param fov field of view around the horizontal axis, i.e. the vertical fov, in radians
	 */
	static createPerspective(fov: number, width: number, height: number, near: number, far: number): Matrix4 {
		const f = 1 / Math.tan(fov / 2);
		const aspect = width / height;
		return new Matrix4(
			f / aspect, 0, 0, 0,
			0, f, 0, 0,
			0, 0, (far + near) / (near - far), 2 * far * near / (near - far),
			0, 0, -1, 0,
		);
	}

	/**
	 * @param position the point where the camera is
	 * @param target a point that will be in the center of the camera's view
	 * @param up a vector that points along the camera's local up axis
	 */
	static createLookAt(position: Vector3, target: Vector3, up: Vector3): Matrix4 {
		const f = target.sub(position).normalized;
		up = up.normalized;
		const s = f.cross(up).normalized;
		const u = s.cross(f).normalized;
		return new Matrix4(
			s.x, u.x, -f.x, 0,
			s.y, u.y, -f.y, 0,
			s.z, u.z, -f.z, 0,
			0, 0, 0, 1,
		)
			.mul(Matrix4.createTranslation(position.negated));
	}

	toArray(): number[] {
		return [
			this.d00, this.d10, this.d20, this.d30,
			this.d01, this.d11, this.d21, this.d31,
			this.d02, this.d12, this.d22, this.d32,
			this.d03, this.d13, this.d23, this.d33,
		];
	}

	mul(other: Matrix4): Matrix4 {
		// TODO manually unroll
		const a = this.toArray();
		const b = other.toArray();
		const result = new Array<number>(16);
		for (let i = 0; i < 4; i++) {
			for (let j = 0; j < 4; j++) {
				result[i * 4 + j] = 0;
				for (let k = 0; k < 4; k++) {
					result[i * 4 + j] += a[i * 4 + k] * b[k * 4 + j];
				}
			}
		}
		return new Matrix4(
			result[0],
			result[1],
			result[2],
			result[3],
			result[4],
			result[5],
			result[6],
			result[7],
			result[8],
			result[9],
			result[10],
			result[11],
			result[12],
			result[13],
			result[14],
			result[15],
		);
	}

	translate(v: Vector3): Matrix4 {
		return Matrix4.createTranslation(v).mul(this);
	}

	scale(v: Vector3): Matrix4 {
		return Matrix4.createScale(v).mul(this);
	}

	/**
	 * @param angle in radians
	 */
	rotate(axis: Vector3, angle: number): Matrix4 {
		return Matrix4.createRotation(axis, angle).mul(this);
	}
}