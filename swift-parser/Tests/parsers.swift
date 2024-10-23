import XCTest
import experiment

class ParsersTest: XCTestCase {
	func testStringSuccess() {
		let parser = string("foo")
		let result = parser.apply(input: "foobar")
		XCTAssertEqual(result, .success(ParseResult(result: "foo", remainder: "bar")))
	}

	func testStringFailure() {
		let parser = string("bar")
		let result = parser.apply(input: "foobar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testMapSuccess() {
		let parser = string("foo")
			.map { result in
				.success(result.count)
			}
		let result = parser.apply(input: "foobar")
		XCTAssertEqual(result, .success(ParseResult(result: 3, remainder: "bar")))
	}

	func testMapParserSucceedsButMapFails() {
		let parser = string("foo")
			.map { result -> Result<Int, ParseError> in
				.failure(ParseError())
			}
		let result = parser.apply(input: "foobar")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testMapParserFailed() {
		let parser = string("foo")
			.map { result in
				.success(result.count)
			}
		let result = parser.apply(input: "barfoo")
		XCTAssertEqual(result, .failure(ParseError()))
	}

	func testRegexSuccess() {
		let parser = regex(/[0-9]+/)
		let result = parser.apply(input: "123abc")
		XCTAssertEqual(result, .success(ParseResult(result: "123", remainder: "abc")))
	}

	func testRegexFailed() {
		let parser = regex(/[0-9]+/)
		let result = parser.apply(input: "abc")
		XCTAssertEqual(result, .failure(ParseError()))
	}
}
