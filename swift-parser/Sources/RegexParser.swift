class RegexParser: Parser {
	typealias T = Swift.Substring

	private let regex: Regex<AnyRegexOutput>
	private let description: String

	init(_ regex: Regex<AnyRegexOutput>, description: String) {
		self.regex = regex
		self.description = description
	}

	func eval(input: Substring) -> Result<Swift.Substring> {
		if input.string.isEmpty {
			return .failure(.endOfInput)
		}
		do {
			if let m = try regex.prefixMatch(in: input.string) {
				let (text, remainder) = input.split(length: m.0.count)
				return .success(.init(text: text, value: text.string, remainder: remainder))
			} else {
				return .failure(.expected("\(description)", input.location))
			}
		} catch {
			return .failure(.unknown("\(error)", input.location))
		}
	}
}

public func regex(_ regex: Regex<AnyRegexOutput>, description: String) -> some Parser<
	Swift.Substring
> {
	RegexParser(regex, description: description)
}

public func regex(_ regex: String, description: String? = nil) throws -> some Parser<
	Swift.Substring
> {
	try RegexParser(Regex(regex), description: description ?? regex)
}
