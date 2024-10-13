export interface SuccessResult<T> {
	success: true;
	result: T;
	remainder: string;
}

export interface FailureResult {
	success: false;
	reason: unknown;
}

export type Result<T> = SuccessResult<T> | FailureResult;

export class ExpectedError extends Error {
	constructor(
		readonly expected: string,
		readonly received: string,
	) {
		super(`expected: ${expected}, received: ${received}`);
	}
}

export class EndOfInput extends ExpectedError {
	constructor(expected: string) {
		super(expected, "<end of input>");
	}
}

export class MultiError extends Error {
	constructor(readonly errors: ReadonlyArray<unknown>) {
		const errorStr = errors
			.map(e => {
				if (e instanceof Error) {
					return e.message;
				}
				return `${e}`;
			})
			.join(", ");
		super(`errors: [${errorStr}]`);
	}
}

export interface Parser<T> {
	(input: string): Result<T>;

	map<R>(f: (result: T) => R): Parser<R>;
}

export function string(s: string): Parser<string> {
	return newParser(input => {
		if (input.startsWith(s)) {
			return {
				success: true,
				result: s,
				remainder: input.substring(s.length),
			};
		} else if (input.length < s.length) {
			return {
				success: false,
				reason: new EndOfInput(s),
			};
		} else {
			return {
				success: false,
				reason: new ExpectedError(
					s,
					input.substring(0, s.length),
				),
			};
		}
	});
}

export function charRange(start: string, end: string): Parser<string> {
	const expected = `${start}..${end}`;
	if (start.length !== 1 || end.length !== 1) {
		throw new Error(`arguments should be single characters, got ${expected}`);
	}
	return newParser(input => {
		if (input.length === 0) {
			return {
				success: false,
				reason: new EndOfInput(expected),
			};
		}
		const next = input[0];
		if (next < start || next > end) {
			return {
				success: false,
				reason: new ExpectedError(expected, next),
			};
		}
		return {
			success: true,
			result: next,
			remainder: input.substring(1),
		};
	});
}

export function seq<T extends unknown[]>(...parsers: { [K in keyof T]: Parser<T[K]> }): Parser<T> {
	return newParser(input => {
		let remainder = input;
		const results = [] as unknown as T;
		for (const p of parsers) {
			let intermediateResult = p(remainder);
			if (!intermediateResult.success) {
				return intermediateResult;
			}
			results.push(intermediateResult.result);
			remainder = intermediateResult.remainder;
		}
		return {
			success: true,
			result: results,
			remainder,
		};
	});
}

export function any<T>(...parsers: Parser<T>[]): Parser<T> {
	return newParser(input => {
		const errors: unknown[] = [];
		for (const p of parsers) {
			const result = p(input);
			if (result.success) {
				return result;
			}
			errors.push(result.reason);
		}
		return {
			success: false,
			reason: new MultiError(errors),
		};
	});
}

export function count<T>(parser: Parser<T>, count: number): Parser<T[]> {
	if (count !== (count | 0) || count < 1) {
		throw new Error(`must provide a positive integer for count, got ${count}`);
	}
	return newParser(input => {
		let remainder = input;
		const results: T[] = [];
		for (let i = 0; i < count; i++) {
			const intermediateResult = parser(remainder);
			if (!intermediateResult.success) {
				return intermediateResult;
			}
			results.push(intermediateResult.result);
			remainder = intermediateResult.remainder;
		}
		return {
			success: true,
			result: results,
			remainder,
		};
	});
}

export function many0<T>(parser: Parser<T>): Parser<T[]> {
	return newParser(input => {
		let remainder = input;
		const results: T[] = [];
		while (true) {
			const intermediateResult = parser(remainder);
			// failure doesn't mean failure, we're just done trying
			if (!intermediateResult.success) {
				break;
			}
			// escape hatch if we have a weird parser that isn't making progress
			if (intermediateResult.remainder === remainder) {
				break;
			}
			results.push(intermediateResult.result);
			remainder = intermediateResult.remainder;
		}
		return {
			success: true,
			result: results,
			remainder,
		};
	});
}

export function many1<T>(parser: Parser<T>): Parser<T[]> {
	const parser0 = many0(parser);
	return newParser(input => {
		const first = parser(input);
		if (!first.success) {
			return first;
		}
		const rest = parser0(first.remainder);
		if (!rest.success) {
			return {
				success: true,
				result: [first.result],
				remainder: first.remainder,
			};
		}
		return {
			success: true,
			result: [first.result, ...rest.result],
			remainder: rest.remainder,
		};
	});
}

export function optional<T>(parser: Parser<T>): Parser<T | null> {
	return newParser(input => {
		const result = parser(input);
		if (result.success) {
			return {
				success: true,
				result: result.result,
				remainder: result.remainder,
			};
		}
		return {
			success: true,
			result: null,
			remainder: input,
		};
	});
}

export function delimited<T1, T2, T3>(parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>): Parser<T2> {
	return seq(parser1, parser2, parser3)
		.map(([, result,]) => result);
}

export function preceded<T1, T2>(parser1: Parser<T1>, parser2: Parser<T2>): Parser<T2> {
	return seq(parser1, parser2)
		.map(([, result]) => result);
}

export function terminated<T1, T2>(parser1: Parser<T1>, parser2: Parser<T2>): Parser<T1> {
	return seq(parser1, parser2)
		.map(([result,]) => result);
}

export type DeferSetter<T> = (p: Parser<T>) => void;

export function defer<T>(): [DeferSetter<T>, Parser<T>] {
	let parser: Parser<T> | null = null;
	const setter: DeferSetter<T> = p => {
		parser = p;
	};
	return [
		setter,
		newParser(input => {
			if (!parser) {
				throw new Error("deferred parser used before it was initialized");
			}
			return parser(input);
		})
	];
}

function newParser<T>(f: (input: string) => Result<T>): Parser<T> {
	return Object.assign(
		f,
		{
			map: <R>(g: (result: T) => R) => {
				return newParser(input => {
					const intermediateResult = f(input);
					if (intermediateResult.success) {
						return {
							success: true,
							result: g(intermediateResult.result),
							remainder: intermediateResult.remainder,
						};
					} else {
						return intermediateResult;
					}
				});
			},
		}
	);
}