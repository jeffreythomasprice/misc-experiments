import Experiment
import Testing

@Suite struct LocationTests {
	@Test(arguments: [
		(
			Location(line: 0, column: 0),
			Character("a"),
			Location(line: 0, column: 1)
		),
		(
			Location(line: 2, column: 3),
			Character("b"),
			Location(line: 2, column: 4)
		),
		(
			Location(line: 0, column: 0),
			Character("\r"),
			Location(line: 0, column: 1)
		),
		(
			Location(line: 4, column: 6),
			Character("\n"),
			Location(line: 5, column: 0)
		),
	])
	func advance(location: Location, c: Character, expected: Location) {
		#expect(location.advance(c: c) == expected)
	}
}

@Suite struct SubstringTests {
	@Test(arguments: [
		(
			Substring(string: "abc", location: Location(line: 0, column: 0)),
			0,
			(
				Substring(string: "", location: Location(line: 0, column: 0)),
				Substring(string: "abc", location: Location(line: 0, column: 0))
			)
		),
		(
			Substring(string: "abc", location: Location(line: 0, column: 0)),
			1,
			(
				Substring(string: "a", location: Location(line: 0, column: 0)),
				Substring(string: "bc", location: Location(line: 0, column: 1))
			)
		),
		(
			Substring(string: "abc", location: Location(line: 0, column: 0)),
			2,
			(
				Substring(string: "ab", location: Location(line: 0, column: 0)),
				Substring(string: "c", location: Location(line: 0, column: 2))
			)
		),
		(
			Substring(string: "abc", location: Location(line: 0, column: 0)),
			3,
			(
				Substring(string: "abc", location: Location(line: 0, column: 0)),
				Substring(string: "", location: Location(line: 0, column: 3))
			)
		),
		(
			Substring(string: "abc", location: Location(line: 0, column: 0)),
			4,
			(
				Substring(string: "abc", location: Location(line: 0, column: 0)),
				Substring(string: "", location: Location(line: 0, column: 3))
			)
		),
		(
			Substring(string: "foo\nbar\nbaz", location: Location(line: 2, column: 5)),
			2,
			(
				Substring(string: "fo", location: Location(line: 2, column: 5)),
				Substring(string: "o\nbar\nbaz", location: Location(line: 2, column: 7))
			)
		),
		(
			Substring(string: "foo\nbar\nbaz", location: Location(line: 2, column: 5)),
			7,
			(
				Substring(string: "foo\nbar", location: Location(line: 2, column: 5)),
				Substring(string: "\nbaz", location: Location(line: 3, column: 3))
			)
		),
		(
			Substring(string: "foo\nbar\nbaz", location: Location(line: 2, column: 5)),
			8,
			(
				Substring(string: "foo\nbar\n", location: Location(line: 2, column: 5)),
				Substring(string: "baz", location: Location(line: 4, column: 0))
			)
		),
	])
	func split(input: Substring, length: Int, expected: (Substring, Substring)) {
		#expect(input.split(length: length) == expected)
	}
}
