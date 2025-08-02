import { literal, Parser, regex, seq } from "./parser";

export function number(): Parser<number> {
	return regex(/^-?(?:\d+\.?\d*|\.\d+)/)
		.map(str => parseFloat(str));
}

export function identifier(): Parser<string> {
	return regex(/^[a-zA-Z_][a-zA-Z0-9_]*/)
}

export function label(): Parser<string> {
	return seq(
		identifier(),
		literal(":"),
	)
		.map(([name]) => name);
}
