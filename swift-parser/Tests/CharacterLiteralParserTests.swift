import Experiment
import Testing

@Suite struct CharacterLiteralParserTests {
	@Test(arguments: [
		(
			"a",
			"a",
			Result.success(
				Ok(
					text: Substring(string: "a", location: .init(line: 0, column: 0)),
					value: "a",
					remainder: Substring(string: "", location: .init(line: 0, column: 1))
				))
		),
		(
			"a",
			"abc",
			Result.success(
				Ok(
					text: Substring(string: "a", location: .init(line: 0, column: 0)),
					value: "a",
					remainder: Substring(string: "bc", location: .init(line: 0, column: 1))
				))
		),
		(
			"a",
			"",
			Result.failure(.endOfInput)
		),
		(
			"a",
			"bc",
			Result.failure(.expected("a", Location(line: 0, column: 0)))
		),
	])
	func eval(c: Character, input: String, expected: Experiment.Result<Character>) {
		let p = char(c)
		let result = p.eval(input: Substring(string: input, location: .init(line: 0, column: 0)))
		#expect(result == expected)
	}
}
