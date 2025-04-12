import Experiment
import Testing

@Suite struct StringLiteralParserTests {
	@Test(arguments: [
		(
			"abc",
			"abcdefgh",
			Result.success(
				Ok(
					text: Substring(string: "abc", location: .init(line: 0, column: 0)),
					value: "abc",
					remainder: Substring(string: "defgh", location: .init(line: 0, column: 3))
				))
		),
		(
			"abc",
			"bcdefgh",
			Result.failure(.expected("abc", Location(line: 0, column: 0)))
		),
		(
			"abc",
			"bc",
			Result.failure(.endOfInput)
		),
	])
	func eval(s: String, input: String, expected: Experiment.Result<Swift.Substring>) {
		let p = string(s)
		let result = p.eval(input: Substring(string: input, location: .init(line: 0, column: 0)))
		#expect(result == expected)
	}
}
