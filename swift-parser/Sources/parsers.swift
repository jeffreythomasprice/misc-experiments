public struct ParseResult<T> {
	public let result: T
	public let remainder: Substring

	public init(result: T, remainder: Substring) {
		self.result = result
		self.remainder = remainder
	}
}

extension ParseResult: Equatable where T: Equatable {}

public struct ParseError: Error, Equatable {
	public init() {}
}

public protocol Parser<T> {
	associatedtype T

	func apply(input: String) -> Result<ParseResult<T>, ParseError>
}

struct MapParser<SourceType, ReturnType>: Parser {
	typealias T = ReturnType

	let p: any Parser<SourceType>
	let f: (SourceType) -> Result<ReturnType, ParseError>

	func apply(input: String) -> Result<ParseResult<ReturnType>, ParseError> {
		return switch p.apply(input: input) {
		case .success(let result):
			switch f(result.result) {
			case .success(let newResult):
				.success(
					ParseResult(
						result: newResult,
						remainder: result.remainder
					))

			case .failure(let e):
				.failure(e)
			}

		case .failure(let e):
			.failure(e)
		}
	}
}

extension Parser {
	public func map<R>(f: @escaping (T) -> Result<R, ParseError>) -> any Parser<R> {
		MapParser(p: self, f: f)
	}
}

struct StringParser: Parser {
	typealias T = String

	let s: String

	func apply(input: String) -> Result<ParseResult<String>, ParseError> {
		if input.starts(with: s) {
			.success(
				ParseResult(
					result: s,
					remainder: input.dropFirst(s.count)
				))
		} else {
			.failure(ParseError())
		}
	}
}

public func string(_ s: String) -> any Parser<String> {
	StringParser(s: s)
}

struct RegexParser: Parser {
	typealias T = Substring

	let r: Regex<Substring>

	func apply(input: String) -> Result<ParseResult<Substring>, ParseError> {
		switch try? r.prefixMatch(in: input) {
		case .none:
			.failure(ParseError())
		case .some(let result):
			.success(
				ParseResult(result: result.output, remainder: input[result.output.endIndex...]))
		}
	}
}

public func regex(_ r: Regex<Substring>) -> any Parser<Substring> {
	RegexParser(r: r)
}

// TODO seq2
// TODO seq3
// TODO any
// TODO skip
// TODO bracketed
// TODO repeat
// TODO many0
// TODO many1
// TODO optional
