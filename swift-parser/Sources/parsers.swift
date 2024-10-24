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

	func apply(input: Substring) -> Result<ParseResult<T>, ParseError>
}

struct MapParser<SourceType, ReturnType>: Parser {
	typealias T = ReturnType

	let p: any Parser<SourceType>
	let f: (SourceType) -> Result<ReturnType, ParseError>

	func apply(input: Substring) -> Result<ParseResult<ReturnType>, ParseError> {
		p(input: input)
			.flatMap { result in
				f(result.result).map { newResult in
					ParseResult(result: newResult, remainder: result.remainder)
				}
			}
	}
}

extension Parser {
	public func apply(input: String) -> Result<ParseResult<T>, ParseError> {
		self.apply(input: Substring(input))
	}

	public func callAsFunction(input: Substring) -> Result<ParseResult<T>, ParseError> {
		self.apply(input: input)
	}

	public func map<R>(f: @escaping (T) -> Result<R, ParseError>) -> any Parser<R> {
		MapParser(p: self, f: f)
	}
}

struct StringParser: Parser {
	typealias T = String

	let s: String

	func apply(input: Substring) -> Result<ParseResult<String>, ParseError> {
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

	func apply(input: Substring) -> Result<ParseResult<Substring>, ParseError> {
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

struct Seq2Parser<T1, T2>: Parser {
	typealias T = (T1, T2)

	let p1: any Parser<T1>
	let p2: any Parser<T2>

	func apply(input: Substring) -> Result<ParseResult<(T1, T2)>, ParseError> {
		do {
			let r1 = try p1(input: input).get()
			let r2 = try p2(input: r1.remainder).get()
			return .success(
				ParseResult(
					result: (r1.result, r2.result),
					remainder: r2.remainder
				))
		} catch {
			return .failure(error)
		}
	}
}

public func seq2<T1, T2>(
	_ p1: any Parser<T1>,
	_ p2: any Parser<T2>
) -> any Parser<(T1, T2)> {
	Seq2Parser(p1: p1, p2: p2)
}

struct Seq3Parser<T1, T2, T3>: Parser {
	typealias T = (T1, T2, T3)

	let p1: any Parser<T1>
	let p2: any Parser<T2>
	let p3: any Parser<T3>

	func apply(input: Substring) -> Result<ParseResult<(T1, T2, T3)>, ParseError> {
		do {
			let r1 = try p1(input: input).get()
			let r2 = try p2(input: r1.remainder).get()
			let r3 = try p3(input: r2.remainder).get()
			return .success(
				ParseResult(
					result: (r1.result, r2.result, r3.result),
					remainder: r3.remainder
				))
		} catch {
			return .failure(error)
		}
	}
}

public func seq3<T1, T2, T3>(
	_ p1: any Parser<T1>,
	_ p2: any Parser<T2>,
	_ p3: any Parser<T3>
) -> any Parser<(T1, T2, T3)> {
	Seq3Parser(p1: p1, p2: p2, p3: p3)
}

struct AnyParser<T>: Parser {
	let parsers: [any Parser<T>]

	func apply(input: Substring) -> Result<ParseResult<T>, ParseError> {
		for p in parsers {
			if case .success(let result) = p(input: input) {
				return .success(result)
			}
		}
		return .failure(ParseError())
	}
}

public func any<T>(_ parsers: any Parser<T>...) -> any Parser<T> {
	AnyParser(parsers: parsers)
}

public func skip<T1, T2>(_ p1: any Parser<T1>, _ p2: any Parser<T2>) -> any Parser<T2> {
	seq2(p1, p2).map { result in .success(result.1) }
}

public func bracketed<T1, T2, T3>(_ p1: any Parser<T1>, _ p2: any Parser<T2>, _ p3: any Parser<T3>)
	-> any Parser<T2>
{
	seq3(p1, p2, p3).map { result in .success(result.1) }
}

// TODO repeat
// TODO many0
// TODO many1
// TODO optional
