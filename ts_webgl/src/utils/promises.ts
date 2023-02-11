export type MaybePromise<T> = T | PromiseLike<T>;

export function isPromiseLike<T>(x: MaybePromise<T>): x is PromiseLike<T> {
	return !!x &&
		typeof x === "object" &&
		typeof (x as PromiseLike<T>).then === "function";
}

export async function awaitIfNeeded<T>(x: MaybePromise<T>): Promise<T> {
	if (isPromiseLike(x)) {
		return await x;
	} else {
		return x;
	}
}
