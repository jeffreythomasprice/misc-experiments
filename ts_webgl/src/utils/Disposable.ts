export abstract class Disposable {
	private isDisposed = false;

	dispose() {
		if (this.isDisposed) {
			return;
		}
		this.isDisposed = true;
		this.disposeImpl();
	}

	protected abstract disposeImpl(): void;
}