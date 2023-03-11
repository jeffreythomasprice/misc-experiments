export function clamp(value: number, min: number, max: number): number {
	if (value < min) {
		return min;
	}
	if (value > max) {
		return max;
	}
	return value;
}

export function wrap(value: number, min: number, max: number): number {
	return ((value - min) % (max - min)) + min;
}