class CharacterLiteralParser: Parser {
	typealias T = Character

	private let c: Character

	init(_ c: Character) {
		self.c = c
	}

	func eval(input: Substring) -> Result<Character> {
		if input.string.isEmpty {
			return .failure(.endOfInput)
		}
		if !input.string.hasPrefix(String(c)) {
			return .failure(.expected("\(c)", input.location))
		}
		let (text, remainder) = input.split(length: 1)
		return .success(.init(text: text, value: c, remainder: remainder))
	}
}

public func char(_ c: Character) -> some Parser<Character> {
	CharacterLiteralParser(c)
}
