export const API_BASE_URL = assertTruthy("API_BASE_URL", process.env.API_BASE_URL);
export const WS_BASE_URL = assertTruthy("WS_BASE_URL", process.env.WS_BASE_URL);

function assertTruthy<T>(key: string, value: T | undefined | null): T {
	if (!value) {
		throw new Error(`must provide ${key}`);
	}
	return value;
}