export class DirtyRegionTracker {
	private dirty: {
		min: number;
		max: number;
	} | null = null;

	add(x: number): void;
	add(min: number, max: number): void;
	add(min: number, max?: number): void {
		if (typeof max === "number") {
			if (this.dirty) {
				this.dirty.min = Math.min(this.dirty.min, min);
				this.dirty.max = Math.max(this.dirty.max, max);
			} else {
				this.dirty = { min, max };
			}
		} else {
			this.add(min, min);
		}
	}

	clear(): DirtyRegionTracker["dirty"] {
		const result = this.dirty;
		this.dirty = null;
		return result;
	}
}