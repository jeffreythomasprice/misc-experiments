export class InputLocation {
	constructor(
		public readonly line: number,
		public readonly column: number,
	) { }

	toString(): string {
		return `${this.line}:${this.column}`;
	}

	advance(s: string): InputLocation {
		let line = this.line;
		let column = this.column;
		for (let c of s) {
			if (c === '\n') {
				line++;
				column = 0;
			} else {
				column++;
			}
		}
		return new InputLocation(line, column);
	}
}

export interface Input {
	readonly text: string;
	readonly location: InputLocation;
}

export interface ParseSuccess<T> {
	readonly value: T;
	readonly remainder: Input;
}

export class ParseError extends Error {
	constructor(
		public readonly location: InputLocation,
		public readonly message: string
	) {
		super(`${location}: ${message}`);
	}
}

export class ExpectedError extends ParseError {
	constructor(
		public readonly location: InputLocation,
		public readonly expected: string
	) {
		super(location, `expected ${expected}`);
	}
}

export abstract class Parser<T> {
	/**
	 * 
	 * @param input 
	 * @throws ParseError if it doesn't match
	 */
	abstract parse(input: Input): ParseSuccess<T>;

	map<R>(f: (value: T) => R): Parser<R> {
		const parent = this;
		return new (class extends Parser<R> {
			parse(input: Input): ParseSuccess<R> {
				const result = parent.parse(input);
				return {
					value: f(result.value),
					remainder: result.remainder
				};
			}
		})();
	}
}

class LiteralParser extends Parser<string> {
	constructor(
		private readonly literal: string,
		private readonly caseSensitive: boolean
	) {
		super();
	}

	parse(input: Input): ParseSuccess<string> {
		const result = input.text.substring(0, this.literal.length);
		let success: boolean;
		if (this.caseSensitive) {
			success = result === this.literal;
		} else {
			success = result.localeCompare(this.literal, undefined, { sensitivity: 'base' }) === 0;
		}
		if (!success) {
			throw new ExpectedError(input.location, this.literal);
		}
		return {
			value: result,
			remainder: {
				text: input.text.slice(result.length),
				location: input.location.advance(result)
			}
		};
	}
}

export function literal(literal: string, caseSensitive: boolean = true): Parser<string> {
	return new LiteralParser(literal, caseSensitive);
}

class RegexParser extends Parser<string> {
	constructor(private readonly regex: RegExp) {
		super();
	}

	parse(input: Input): ParseSuccess<string> {
		const match = this.regex.exec(input.text);
		if (!match || match.index !== 0) {
			throw new ExpectedError(input.location, `${this.regex}`);
		}
		const result = match[0];
		return {
			value: result,
			remainder: {
				text: input.text.slice(result.length),
				location: input.location.advance(result),
			}
		};
	}
}

export function regex(regex: RegExp): Parser<string> {
	return new RegexParser(regex);
}

class SequenceParser<T extends unknown[]> extends Parser<T> {
	constructor(private readonly parsers: Parser<T[number]>[]) {
		super();
	}

	parse(input: Input): ParseSuccess<T> {
		const values: T = [] as unknown as T;
		let currentInput = input;

		for (const parser of this.parsers) {
			const result = parser.parse(currentInput);
			values.push(result.value);
			currentInput = result.remainder;
		}

		return {
			value: values,
			remainder: currentInput
		};
	}
}

export function seq<T extends unknown[]>(...parsers: { [K in keyof T]: Parser<T[K]> }): Parser<T> {
	return new SequenceParser(parsers);
}

class OneOfParser<T> extends Parser<T> {
	constructor(private readonly parsers: Parser<T>[]) {
		super();
	}

	parse(input: Input): ParseSuccess<T> {
		const errors: string[] = [];
		for (const parser of this.parsers) {
			try {
				return parser.parse(input);
			} catch (e) {
				if (e instanceof Error) {
					errors.push(e.message);
				} else {
					errors.push(`${e}`);
				}
			}
		}
		throw new ParseError(
			input.location,
			`all possibilities failed: ${errors.join(', ')}`
		);
	}
}

export function oneOf<T>(...parsers: Parser<T>[]): Parser<T> {
	return new OneOfParser(parsers);
}

class DeferredParser<T> extends Parser<T> {
	private parser: Parser<T> | null = null;

	constructor() {
		super();
	}

	set(parser: Parser<T>): void {
		if (this.parser) {
			throw new Error("Deferred parser already initialized");
		}
		this.parser = parser;
	}

	parse(input: Input): ParseSuccess<T> {
		if (!this.parser) {
			throw new Error("Deferred parser not initialized");
		}
		return this.parser.parse(input);
	}
}

export function defer<T>(): [Parser<T>, (parser: Parser<T>) => void] {
	const result = new DeferredParser<T>();
	return [result, (parser: Parser<T>) => result.set(parser)];
}