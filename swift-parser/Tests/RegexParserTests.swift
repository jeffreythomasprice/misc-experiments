import Experiment
import Testing

@Suite struct RegexParserTests {
	@Test(arguments: [
		(
			"[0-9]+",
			"123abc",
			Result.success(
				Ok(
					text: Substring(string: "123", location: .init(line: 0, column: 0)),
					value: "123",
					remainder: Substring(string: "abc", location: .init(line: 0, column: 3))
				))
		),
		(
			"[0-9]+",
			"abc123",
			Result.failure(.expected("[0-9]+", Location(line: 0, column: 0)))
		),
		(
			"[0-9]+",
			"",
			Result.failure(.endOfInput)
		),
	])
	func eval(
		r: String, input: String, expected: Experiment.Result<Swift.Substring>
	) {
		let p = try! regex(r)
		let result = p.eval(input: Substring(string: input, location: .init(line: 0, column: 0)))
		#expect(result == expected)
	}
}
