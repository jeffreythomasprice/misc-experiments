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

struct RangeParser<T>: Parser {
	typealias T = [T]

	let p: any Parser<T>
	let contains: (Int) -> Bool
	let upperBound: Int?

	func apply(input: Substring) -> Result<ParseResult<[T]>, ParseError> {
		// all results matched so far
		var results: [T] = []
		var remainder = input

		// the last successful set of results
		// nil means we have no such results to return, but empty list means zero results counts as a success
		var validResult: ParseResult<[T]>? =
			if contains(0) {
				ParseResult(result: [], remainder: input)
			} else {
				nil
			}

		loop: while true {
			// if we have enough results and adding one more would put us past our upper bound we can stop here
			if let upperBound = upperBound {
				if results.count + 1 > upperBound {
					break
				}
			}

			// try to find a new match
			switch p(input: remainder) {
			case .success(let result):
				// we got a success on no input, so we're not actually making any more progress
				if result.remainder == remainder {
					break loop
				}

				// we have a success, keep it
				results.append(result.result)
				remainder = result.remainder

				// if this is a good number of results remember that for later
				if contains(results.count) {
					validResult = ParseResult(result: results, remainder: remainder)
				}

			case .failure(_):
				// we got a failure, so we're done checking
				break loop
			}
		}

		// if we have a good set of results we can succeed on that, otherwise we failed
		return switch validResult {
		case .none:
			.failure(ParseError())

		case .some(let r):
			.success(r)
		}
	}
}

// x...
public func range<T>(_ p: any Parser<T>, _ r: PartialRangeFrom<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: nil)
}

// ...x
public func range<T>(_ p: any Parser<T>, _ r: PartialRangeThrough<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: r.upperBound + 1)
}

// ..<x
public func range<T>(_ p: any Parser<T>, _ r: PartialRangeUpTo<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: r.upperBound)
}

// x...y
public func range<T>(_ p: any Parser<T>, _ r: ClosedRange<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: r.upperBound + 1)
}

// x..<y
public func range<T>(_ p: any Parser<T>, _ r: Range<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: r.upperBound)
}

// stride(from: x, through: y, by: z)
public func range<T>(_ p: any Parser<T>, _ r: StrideThrough<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: r.suffix(1)[0])
}

// stride(from: x, to: y, by: z)
public func range<T>(_ p: any Parser<T>, _ r: StrideTo<Int>) -> any Parser<[T]> {
	RangeParser(p: p, contains: r.contains, upperBound: r.suffix(1)[0])
}

// TODO many0
// TODO many1
// TODO optional
