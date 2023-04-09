import { Readable } from "stream";
import { Observable } from "rxjs";

export interface TokenMatcher<TokenName> {
	readonly name: TokenName;
	match(input: string): Token<TokenName> | false;
}

export interface Token<TokenName> {
	readonly name: TokenName;
	readonly value: string;
	readonly matcher: TokenMatcher<TokenName>;
	// TODO include line and column
}

export class StringLiteralTokenMatcher<TokenName> implements TokenMatcher<TokenName> {
	constructor(
		public readonly name: TokenName,
		public readonly value: string,
	) { }

	match(input: string): false | Token<TokenName> {
		if (input.substring(0, this.value.length) === this.value) {
			return {
				name: this.name,
				value: this.value,
				matcher: this
			};
		}
		return false;
	}
}

export class RegexTokenMatcher<TokenName> implements TokenMatcher<TokenName> {
	constructor(
		public readonly name: TokenName,
		public readonly regex: RegExp,
	) { }

	match(input: string): false | Token<TokenName> {
		const result = this.regex.exec(input);
		if (!result) {
			return false;
		}
		if (result.index !== 0) {
			return false;
		}
		return {
			name: this.name,
			value: result[0],
			matcher: this
		};
	}
}

export interface Tokenizer<TokenName> {
	tokenize(input: Readable | Buffer | string): Observable<Token<TokenName>>;
}

export namespace Tokenizer {
	export class Builder<TokenName> {
		private readonly matchers: TokenMatcher<TokenName>[] = [];
		private encoding: BufferEncoding = "utf8";
		private peekLimit: number = 1024;

		add(matcher: TokenMatcher<TokenName>): Builder<TokenName>;
		add(name: TokenName, value: string | RegExp): Builder<TokenName>;
		add(matcherOrName: TokenMatcher<TokenName> | TokenName, value?: string | RegExp): Builder<TokenName> {
			if (typeof value !== "undefined") {
				const name = matcherOrName as TokenName;
				if (typeof value === "string") {
					return this.add(new StringLiteralTokenMatcher(name, value));
				}
				return this.add(new RegexTokenMatcher(name, value));
			}
			const matcher = matcherOrName as TokenMatcher<TokenName>;
			this.matchers.push(matcher);
			return this;
		}

		build(): Tokenizer<TokenName> {
			return new TokenizerImpl([...this.matchers], this.encoding, this.peekLimit);
		}
	}
}

class TokenizerImpl<TokenName> implements Tokenizer<TokenName> {
	constructor(
		public readonly matchers: ReadonlyArray<TokenMatcher<TokenName>>,
		public readonly encoding: BufferEncoding,
		public readonly peekLimit: number,
	) { }

	tokenize(input: string | Readable | Buffer): Observable<Token<TokenName>> {
		return new Observable((subscriber) => {
			if (typeof input === "string") {
				input = Buffer.from(input, this.encoding);
			}
			if (Buffer.isBuffer(input)) {
				input = Readable.from([input]);
			}

			let buffer = Buffer.alloc(0);

			const emit = (): boolean => {
				const input = buffer.toString(this.encoding);
				for (const matcher of this.matchers) {
					const result = matcher.match(input);
					if (!result) {
						continue;
					}
					const matchedBytes = Buffer.from(result.value, this.encoding).length;
					buffer = buffer.subarray(matchedBytes);
					subscriber.next(result);
					return true;
				}
				return false;
			};

			input.on("data", (data) => {
				if (typeof data === "string") {
					data = Buffer.from(data, this.encoding);
				}
				if (!Buffer.isBuffer(data)) {
					throw new Error(`expected string or buffer, got ${typeof data}`);
				}
				buffer = Buffer.concat([buffer, data]);
				while (buffer.length >= this.peekLimit) {
					if (!emit()) {
						break;
					}
				}
			});

			input.on("end", () => {
				while (buffer.length > 0) {
					if (!emit()) {
						break;
					}
				}
				if (buffer.length > 0) {
					const bufferStr = buffer.toString(this.encoding);
					// TODO include line and column in error message, custom Error
					if (bufferStr.length >= 1) {
						subscriber.error(new Error(`unrecognized "${bufferStr[0]}"`));
					} else {
						subscriber.error(new Error(`unrecognized ${buffer[0].toString(16).padStart(2, "0")}`));
					}
				}
				subscriber.complete();
			});

			input.on("error", (e) => {
				subscriber.error(e);
			});
		});
	}
}
