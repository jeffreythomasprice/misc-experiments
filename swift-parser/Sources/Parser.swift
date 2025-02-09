public struct Location: Sendable, Equatable {
	let line: Int
	let column: Int

	public init(line: Int, column: Int) {
		self.line = line
		self.column = column
	}

	public func advance(c: Character) -> Location {
		if c == "\n" {
			Location(line: line + 1, column: 0)
		} else {
			Location(line: line, column: column + 1)
		}
	}
}

public struct Substring: Sendable, Equatable {
	let string: Swift.Substring
	let location: Location

	public init(string: Swift.Substring, location: Location) {
		self.string = string
		self.location = location
	}

	public init(string: Swift.String, location: Location) {
		self.init(string: Swift.Substring(string), location: location)
	}

	public func split(length: Int) -> (Substring, Substring) {
		var index: Swift.String.Index = string.startIndex
		var locationAtIndex = location
		string.prefix(length).forEach { c in
			index = string.index(after: index)
			locationAtIndex = locationAtIndex.advance(c: c)
		}
		let left =
			if index > string.startIndex {
				string[..<index]
			} else {
				Swift.Substring("")
			}
		let right =
			if index < string.endIndex {
				string[index...]
			} else {
				Swift.Substring("")
			}
		return (
			Substring(string: left, location: location),
			Substring(string: right, location: locationAtIndex)
		)
	}
}

public struct Ok<T> {
	let text: Substring
	let value: T
	let remainder: Substring

	public init(text: Substring, value: T, remainder: Substring) {
		self.text = text
		self.value = value
		self.remainder = remainder
	}
}

extension Ok: Sendable where T: Sendable {}

extension Ok: Equatable where T: Equatable {}

public enum Error: Swift.Error, Equatable {
	case endOfInput
	case expected(String, Location)
	case unknown(String, Location)
}

public typealias Result<T> = Swift.Result<Ok<T>, Error>

public protocol Parser<T> {
	associatedtype T

	func eval(input: Substring) -> Result<T>
}
