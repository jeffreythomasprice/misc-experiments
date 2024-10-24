import XCTest
import experiment

class ParsersTest: XCTestCase {
	func testStringSuccess() {
		let parser = string("foo")
		let result = parser(input: "foobar")
		XCTAssertEqual(result, .success(ParseResult(result: "foo", remainder: "bar")))
	}

	func testStringFailure() {
		let parser = string("bar")
		let result = parser(input: "foobar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testMapSuccess() {
		let parser = string("foo")
			.map { result in
				.success(result.count)
			}
		let result = parser(input: "foobar")
		XCTAssertEqual(result, .success(ParseResult(result: 3, remainder: "bar")))
	}

	func testMapParserSucceedsButMapFails() {
		let parser = string("foo")
			.map { result -> Result<Int, ParseError> in
				.failure(ParseError())
			}
		let result = parser(input: "foobar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testMapParserFailed() {
		let parser = string("foo")
			.map { result in
				.success(result.count)
			}
		let result = parser(input: "barfoo")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testRegexSuccess() {
		let parser = regex(/[0-9]+/)
		let result = parser(input: "123abc")
		XCTAssertEqual(result, .success(ParseResult(result: "123", remainder: "abc")))
	}

	func testRegexFailed() {
		let parser = regex(/[0-9]+/)
		let result = parser(input: "abc")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testSeq2Success() {
		let parser = seq2(string("a"), string("b"))
		switch parser(input: "ab__") {
		case .success(let result):
			XCTAssertEqual(result.result.0, "a")
			XCTAssertEqual(result.result.1, "b")
			XCTAssertEqual(result.remainder, "__")
		case .failure(_):
			XCTAssert(false)
		}
	}

	func testSeq2Failure1() {
		let parser = seq2(string("a"), string("b"))
		switch parser(input: "*b__") {
		case .success(_):
			XCTAssert(false)
		case .failure(_):
			XCTAssert(true)
		}
	}

	func testSeq2Failure2() {
		let parser = seq2(string("a"), string("b"))
		switch parser(input: "a*__") {
		case .success(_):
			XCTAssert(false)
		case .failure(_):
			XCTAssert(true)
		}
	}

	func testSeq3Success() {
		let parser = seq3(string("a"), string("b"), string("c"))
		switch parser(input: "abc__") {
		case .success(let result):
			XCTAssertEqual(result.result.0, "a")
			XCTAssertEqual(result.result.1, "b")
			XCTAssertEqual(result.result.2, "c")
			XCTAssertEqual(result.remainder, "__")
		case .failure(_):
			XCTAssert(false)
		}
	}

	func testSeq3Failure1() {
		let parser = seq3(string("a"), string("b"), string("c"))
		switch parser(input: "*bc__") {
		case .success(_):
			XCTAssert(false)
		case .failure(_):
			XCTAssert(true)
		}
	}

	func testSeq3Failure2() {
		let parser = seq3(string("a"), string("b"), string("c"))
		switch parser(input: "a*c__") {
		case .success(_):
			XCTAssert(false)
		case .failure(_):
			XCTAssert(true)
		}
	}

	func testSeq3Failure3() {
		let parser = seq3(string("a"), string("b"), string("c"))
		switch parser(input: "ab*__") {
		case .success(_):
			XCTAssert(false)
		case .failure(_):
			XCTAssert(true)
		}
	}

	func testAnySuccess1() {
		let parser = any(string("foo"), string("bar"))
		let result = parser(input: "foo__")
		XCTAssertEqual(result, .success(ParseResult(result: "foo", remainder: "__")))
	}

	func testAnySuccess2() {
		let parser = any(string("foo"), string("bar"))
		let result = parser(input: "bar__")
		XCTAssertEqual(result, .success(ParseResult(result: "bar", remainder: "__")))
	}

	func testAnyFailure() {
		let parser = any(string("foo"), string("bar"))
		let result = parser(input: "baz__")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testSkipSuccess() {
		let parser = skip(string("foo"), string("bar"))
		let result = parser(input: "foobar__")
		XCTAssertEqual(result, .success(ParseResult(result: "bar", remainder: "__")))
	}

	func testSkipFailureMissingPrefix() {
		let parser = skip(string("foo"), string("bar"))
		let result = parser(input: "bar__")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testSkipFailureMissingaValue() {
		let parser = skip(string("foo"), string("bar"))
		let result = parser(input: "foo__")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testBracketedSuccess() {
		let parser = bracketed(string("("), string("foo"), string(")"))
		let result = parser(input: "(foo)__")
		XCTAssertEqual(result, .success(ParseResult(result: "foo", remainder: "__")))
	}

	func testBracketedFailureMissingPrefix() {
		let parser = bracketed(string("("), string("foo"), string(")"))
		let result = parser(input: "foo)__")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testBracketedFailureMissingValue() {
		let parser = bracketed(string("("), string("foo"), string(")"))
		let result = parser(input: "()__")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testBracketedFailureMissingSuffix() {
		let parser = bracketed(string("("), string("foo"), string(")"))
		let result = parser(input: "(foo__")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testRangePartialRangeFromFailureNotEnough() {
		let parser = range(string("foo"), 2...)
		let result = parser(input: "foobar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testRangePartialRangeFromSuccess() {
		let parser = range(string("foo"), 2...)
		let result = parser(input: "foofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo"],
					remainder: "bar"
				))
		)
	}

	func testRangePartialRangeThroughSuccessEmpty() {
		let parser = range(string("foo"), ...3)
		let result = parser(input: "bar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: [],
					remainder: "bar"
				))
		)
	}

	func testRangePartialRangeThroughSuccessSome() {
		let parser = range(string("foo"), ...3)
		let result = parser(input: "foofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo", "foo"],
					remainder: "bar"
				))
		)
	}

	func testRangePartialRangeThroughSuccessTooMany() {
		let parser = range(string("foo"), ...3)
		let result = parser(input: "foofoofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo", "foo"],
					remainder: "foobar"
				))
		)
	}

	func testRangePartialRangeUpToSuccessNone() {
		let parser = range(string("foo"), ..<3)
		let result = parser(input: "bar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: [],
					remainder: "bar"
				))
		)
	}

	func testRangePartialRangeUpToSuccessTooMany() {
		let parser = range(string("foo"), ..<3)
		let result = parser(input: "foofoofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo"],
					remainder: "foofoobar"
				))
		)
	}

	func testRangeClosedRangeFailureTooFew() {
		let parser = range(string("foo"), 1...3)
		let result = parser(input: "bar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testRangeClosedRangeSuccess() {
		let parser = range(string("foo"), 1...3)
		let result = parser(input: "foofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo"],
					remainder: "bar"
				))
		)
	}

	func testRangeClosedRangeSuccessTooMany() {
		let parser = range(string("foo"), 1...3)
		let result = parser(input: "foofoofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo", "foo"],
					remainder: "foobar"
				))
		)
	}

	func testRangeRangeFailureTooFew() {
		let parser = range(string("foo"), 1..<3)
		let result = parser(input: "bar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testRangeRangeSuccess() {
		let parser = range(string("foo"), 1..<3)
		let result = parser(input: "foofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo"],
					remainder: "bar"
				))
		)
	}

	func testRangeRangeSuccessTooMany() {
		let parser = range(string("foo"), 1..<3)
		let result = parser(input: "foofoofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo"],
					remainder: "foofoobar"
				))
		)
	}

	func testRangeStrideThroughSuccess() {
		let parser = range(string("foo"), stride(from: 1, through: 3, by: 2))
		let result = parser(input: "foofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo", "foo", "foo"],
					remainder: "bar"
				))
		)
	}

	func testRangeStrideThroughSuccessInBetween() {
		let parser = range(string("foo"), stride(from: 1, through: 3, by: 2))
		let result = parser(input: "foofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo"],
					remainder: "foobar"
				))
		)
	}

	func testRangeStrideToSuccessMinimum() {
		let parser = range(string("foo"), stride(from: 1, to: 3, by: 2))
		let result = parser(input: "foobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo"],
					remainder: "bar"
				))
		)
	}

	func testRangeStrideToSuccessTooMany() {
		let parser = range(string("foo"), stride(from: 1, to: 3, by: 2))
		let result = parser(input: "foofoofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo"],
					remainder: "foofoobar"
				))
		)
	}

	func testRangeStrideToSuccessInBetween() {
		let parser = range(string("foo"), stride(from: 1, to: 3, by: 2))
		let result = parser(input: "foofoobar")
		XCTAssertEqual(
			result,
			.success(
				ParseResult(
					result: ["foo"],
					remainder: "foobar"
				))
		)
	}
}
