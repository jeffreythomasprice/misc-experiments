import type { Observable } from "rxjs";

import type { Token } from "./tokenizer";

export interface ParseMatch<TokenName, ResultType> {
	readonly tokens: ReadonlyArray<Token<TokenName>>;
	readonly remaining: ReadonlyArray<Token<TokenName>>;
	readonly result: ResultType;
}

export abstract class Parser<TokenName, ResultType> {
	abstract parse(input: ReadonlyArray<Token<TokenName>>): ParseMatch<TokenName, ResultType> | false;

	map<OutputResultType>(func: MapFunc<TokenName, ResultType, OutputResultType>): Parser<TokenName, OutputResultType> {
		return new MapResultParser(this, func);
	}
}

export type MapFunc<TokenName, InputResultType, OutputResultType> = (input: InputResultType, tokens: ReadonlyArray<Token<TokenName>>) => OutputResultType;

class MapResultParser<TokenName, InputResultType, OutputResultType> extends Parser<TokenName, OutputResultType> {
	constructor(
		private readonly parser: Parser<TokenName, InputResultType>,
		private readonly func: MapFunc<TokenName, InputResultType, OutputResultType>,
	) {
		super();
	}

	parse(input: readonly Token<TokenName>[]): false | ParseMatch<TokenName, OutputResultType> {
		const result = this.parser.parse(input);
		if (!result) {
			return false;
		}
		const newResult = this.func(result.result, result.tokens);
		return {
			tokens: result.tokens,
			remaining: result.remaining,
			result: newResult
		};
	}
}

class LiteralParser<TokenName> extends Parser<TokenName, string> {
	private readonly expected: ReadonlyArray<TokenName>;

	constructor(...expected: TokenName[]) {
		super();
		this.expected = [...expected];
	}

	parse(input: readonly Token<TokenName>[]): false | ParseMatch<TokenName, string> {
		if (input.length < this.expected.length) {
			return false;
		}
		for (let i = 0; i < this.expected.length; i++) {
			if (input[i].name !== this.expected[i]) {
				return false;
			}
		}
		const tokens = input.slice(0, this.expected.length);
		return {
			tokens,
			remaining: input.slice(this.expected.length),
			result: tokens.reduce(
				(a, b) => a + b.value,
				""
			),
		};
	}
}

export function literal<TokenName>(...expected: TokenName[]): Parser<TokenName, string> {
	return new LiteralParser(...expected);
}

class SequenceParser<TokenName, ResultType> extends Parser<TokenName, ResultType[]> {
	private readonly parsers: ReadonlyArray<Parser<TokenName, ResultType>>;

	constructor(...parsers: Parser<TokenName, ResultType>[]) {
		super();
		this.parsers = [...parsers];
	}

	parse(input: readonly Token<TokenName>[]): false | ParseMatch<TokenName, ResultType[]> {
		let remaining = input;
		const results: ParseMatch<TokenName, ResultType>[] = [];
		for (const p of this.parsers) {
			const result = p.parse(remaining);
			if (!result) {
				return false;
			}
			results.push(result);
			remaining = result.remaining;
		}
		return {
			tokens: results.flatMap((result) => result.tokens),
			remaining,
			result: results.map((result) => result.result)
		};
	}
}

// TODO see if there's a smart typescript way of avoiding lots of manual overloads
export function sequence<TokenName, ResultType0>(
	parsers0: Parser<TokenName, ResultType0>,
): Parser<TokenName, [ResultType0]>;
export function sequence<TokenName, ResultType0, ResultType1>(
	parsers0: Parser<TokenName, ResultType0>,
	parsers1: Parser<TokenName, ResultType1>,
): Parser<TokenName, [ResultType0, ResultType1]>;
export function sequence<TokenName, ResultType0, ResultType1, ResultType2>(
	parsers0: Parser<TokenName, ResultType0>,
	parsers1: Parser<TokenName, ResultType1>,
	parsers2: Parser<TokenName, ResultType2>,
): Parser<TokenName, [ResultType0, ResultType1, ResultType2]>;
export function sequence<TokenName, ResultType>(...parsers: Parser<TokenName, ResultType>[]): Parser<TokenName, ResultType[]>;
export function sequence<TokenName, ResultType>(...parsers: Parser<TokenName, ResultType>[]): Parser<TokenName, ResultType[]> {
	return new SequenceParser(...parsers);
}

class OneOfParser<TokenName, ResultType> extends Parser<TokenName, ResultType> {
	private readonly parsers: ReadonlyArray<Parser<TokenName, ResultType>>;

	constructor(...parsers: Parser<TokenName, ResultType>[]) {
		super();
		this.parsers = [...parsers];
	}

	parse(input: readonly Token<TokenName>[]): false | ParseMatch<TokenName, ResultType> {
		for (const p of this.parsers) {
			const result = p.parse(input);
			if (result) {
				return result;
			}
		}
		return false;
	}
}

// TODO see if there's a smart typescript way of avoiding lots of manual overloads
export function oneOf<TokenName, ResultType0>(
	parsers0: Parser<TokenName, ResultType0>,
): Parser<TokenName, ResultType0>;
export function oneOf<TokenName, ResultType0, ResultType1>(
	parsers0: Parser<TokenName, ResultType0>,
	parsers1: Parser<TokenName, ResultType1>,
): Parser<TokenName, ResultType0 | ResultType1>;
export function oneOf<TokenName, ResultType0, ResultType1, ResultType2>(
	parsers0: Parser<TokenName, ResultType0>,
	parsers1: Parser<TokenName, ResultType1>,
	parsers2: Parser<TokenName, ResultType2>,
): Parser<TokenName, ResultType0 | ResultType1 | ResultType2>;
export function oneOf<TokenName, ResultType>(...parsers: Parser<TokenName, ResultType>[]): Parser<TokenName, ResultType>;
export function oneOf<TokenName, ResultType>(...parsers: Parser<TokenName, ResultType>[]): Parser<TokenName, ResultType> {
	return new OneOfParser(...parsers);
}

export type RepeatOptions = {
	readonly min: number;
} | {
	readonly max: number;
} | {
	readonly min: number;
	readonly max: number;
};

class RepeatParser<TokenName, ResultType> extends Parser<TokenName, ResultType[]> {
	constructor(
		private readonly parser: Parser<TokenName, ResultType>,
		private readonly options: RepeatOptions,
	) {
		super();
	}

	parse(input: readonly Token<TokenName>[]): false | ParseMatch<TokenName, ResultType[]> {
		let remaining = input;
		const results: ParseMatch<TokenName, ResultType>[] = [];
		while (true) {
			if (this.isValidLength(results.length) && !this.isValidLength(results.length + 1)) {
				break;
			}
			const result = this.parser.parse(remaining);
			if (!result) {
				break;
			}
			results.push(result);
			remaining = result.remaining;
		}
		if (!this.isValidLength(results.length)) {
			return false;
		}
		return {
			tokens: results.flatMap((result) => result.tokens),
			remaining,
			result: results.map((result) => result.result),
		};
	}

	private isValidLength(length: number): boolean {
		if ("min" in this.options && length < this.options.min) {
			return false;
		}
		if ("max" in this.options && length > this.options.max) {
			return false;
		}
		return true;
	}
}

export function repeat<TokenName, ResultType>(
	parser: Parser<TokenName, ResultType>,
	options: RepeatOptions,
): Parser<TokenName, ResultType[]> {
	return new RepeatParser(parser, options);
}

export function optional<TokenName, ResultType>(
	parser: Parser<TokenName, ResultType>,
): Parser<TokenName, ResultType[]> {
	return repeat(parser, { min: 0, max: 1 });
}

export function atLeastOne<TokenName, ResultType>(
	parser: Parser<TokenName, ResultType>,
): Parser<TokenName, ResultType[]> {
	return repeat(parser, { min: 1 });
}

export function anyNumberOf<TokenName, ResultType>(
	parser: Parser<TokenName, ResultType>,
): Parser<TokenName, ResultType[]> {
	return repeat(parser, { min: 0 });
}

class DeferredParser<TokenName, ResultType> extends Parser<TokenName, ResultType> {
	private parser: Parser<TokenName, ResultType> | null = null;

	resolve(p: Parser<TokenName, ResultType>) {
		if (this.parser) {
			throw new Error("deferred parser was already resolved");
		}
		this.parser = p;
	}

	parse(input: readonly Token<TokenName>[]): false | ParseMatch<TokenName, ResultType> {
		if (!this.parser) {
			throw new Error("deferred parser wasn't resolved before use");
		}
		return this.parser.parse(input);
	}
}

export function defer<TokenName, ResultType>(): [Parser<TokenName, ResultType>, (p: Parser<TokenName, ResultType>) => void] {
	const result = new DeferredParser<TokenName, ResultType>();
	return [
		result,
		(p) => result.resolve(p)
	];
}

export async function parse<TokenName, ResultType>(
	input: ReadonlyArray<Token<TokenName>> | Observable<Token<TokenName>>,
	parser: Parser<TokenName, ResultType>,
): Promise<ParseMatch<TokenName, ResultType> | false> {
	let inputAsArray: Token<TokenName>[];
	if (Array.isArray(input)) {
		inputAsArray = input;
	} else {
		inputAsArray = await observableToArray(input as Observable<Token<TokenName>>);
	}
	return parser.parse(inputAsArray);
}

async function observableToArray<T>(o: Observable<T>): Promise<T[]> {
	const results: T[] = [];
	await o.forEach((value) => {
		results.push(value);
	});
	return results;
}
