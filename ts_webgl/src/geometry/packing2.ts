import { Aabb2 } from "./Aabb2";
import { Vector2 } from "./Vector2";

export function packAabb2<T>(
	_input: {
		readonly key: T,
		readonly size: Vector2,
	}[],
): {
	readonly key: T,
	readonly bounds: Aabb2,
}[] {
	throw new Error("TODO");
}