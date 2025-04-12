class StringLiteralParser: Parser {
	typealias T = Swift.Substring

	private let string: String

	init(_ string: String) {
		self.string = string
	}

	func eval(input: Substring) -> Result<Swift.Substring> {
		if input.string.count < string.count {
			return .failure(.endOfInput)
		}
		if !input.string.hasPrefix(string) {
			return .failure(.expected("\(string)", input.location))
		}
		let (text, remainder) = input.split(length: string.count)
		return .success(.init(text: text, value: text.string, remainder: remainder))
	}
}

public func string(_ string: String) -> some Parser<Swift.Substring> {
	StringLiteralParser(string)
}
