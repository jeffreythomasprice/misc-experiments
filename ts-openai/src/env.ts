import dotenv from "dotenv";

dotenv.config();

export function assertEnvVar(name: string): string {
	const result = process.env[name];
	if (!result) {
		throw new Error(`expected environment variable: ${name}`);
	}
	return result;
}

export function assertIntEnvVar(name: string): number {
	const s = assertEnvVar(name);
	if (!/^[0-9]+$/.exec(s)) {
		throw new Error(`environment variable doesn't look like an integer: ${name}`);
	}
	return parseInt(s, 10);
}