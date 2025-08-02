import { describe, expect, test } from "bun:test";
import { InputLocation, literal, oneOf, ExpectedError, regex, seq, ParseError, defer } from "./parser";

describe(__filename, () => {
	describe("locations", () => {
		test("no newlines", () => {
			expect(new InputLocation(1, 2).advance("hello"))
				.toEqual(new InputLocation(1, 7));
		});

		test("has newlines", () => {
			expect(new InputLocation(1, 2).advance("hello\nworld\n\nfoo"))
				.toEqual(new InputLocation(4, 3));
		});
	});

	describe(literal.name, () => {
		test("case-sensitive literal, success", () => {
			expect(literal("foo")
				.parse({
					text: "foobar",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "foo",
					remainder: {
						text: "bar",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("case-sensitive literal, failure", () => {
			expect(() => literal("FOO")
				.parse({
					text: "foobar",
					location: new InputLocation(1, 2),
				}))
				.toThrow(new ExpectedError(new InputLocation(1, 2), "FOO"));
		});

		test("case-insensitive literal, success", () => {
			expect(literal("FOO", false)
				.parse({
					text: "foobar",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "foo",
					remainder: {
						text: "bar",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("case-insensitive literal, failure", () => {
			expect(() => literal("bar", false)
				.parse({
					text: "foobar",
					location: new InputLocation(1, 2),
				}))
				.toThrow(new ExpectedError(new InputLocation(1, 2), "bar"));
		});
	});

	describe(regex.name, () => {
		test("regex, success", () => {
			expect(regex(/[0-9]+/)
				.parse({
					text: "123foo",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "123",
					remainder: {
						text: "foo",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("regex, failure", () => {
			expect(() => regex(/[0-9]+/)
				.parse({
					text: "foo123",
					location: new InputLocation(1, 2),
				}))
				.toThrow(new ExpectedError(new InputLocation(1, 2), "/[0-9]+/"));
		});
	});

	describe("map", () => {
		test("success", () => {
			expect(regex(/[0-9]+/)
				.map((value) => parseInt(value, 10))
				.parse({
					text: "123foo",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: 123,
					remainder: {
						text: "foo",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("failure in initial matcher", () => {
			expect(() => regex(/[0-9]+/)
				.map((value) => parseInt(value, 10))
				.parse({
					text: "foo123",
					location: new InputLocation(1, 2),
				}))
				.toThrow(new ExpectedError(new InputLocation(1, 2), "/[0-9]+/"));
		});

		test("failure in map function", () => {
			expect(() => regex(/[0-9]+/)
				.map((value) => {
					if (value === "123") {
						throw new Error("Forced failure in map");
					}
					return parseInt(value, 10);
				})
				.parse({
					text: "123foo",
					location: new InputLocation(0, 0),
				}))
				.toThrow("Forced failure in map");
		});
	});

	describe(seq.name, () => {
		test("success", () => {
			expect(seq(
				literal("foo"),
				regex(/[0-9]+/).map((value) => parseInt(value, 10)),
				literal("bar"),
			).parse({
				text: "foo123bar\nremainder",
				location: new InputLocation(0, 0),
			}))
				.toEqual({
					value: [
						"foo",
						123,
						"bar",
					],
					remainder: {
						text: "\nremainder",
						location: new InputLocation(0, 9),
					},
				});
		});

		test("failure on first parser", () => {
			expect(() => seq(
				literal("foo"),
				regex(/[0-9]+/).map((value) => parseInt(value, 10)),
				literal("bar"),
			).parse({
				text: "FOO123bar\nremainder",
				location: new InputLocation(0, 0),
			}))
				.toThrow(new ExpectedError(new InputLocation(0, 0), "foo"));
		});

		test("failure on second parser", () => {
			expect(() => seq(
				literal("foo"),
				regex(/[0-9]+/).map((value) => parseInt(value, 10)),
				literal("bar"),
			).parse({
				text: "foobar\nremainder",
				location: new InputLocation(0, 0),
			}))
				.toThrow(new ExpectedError(new InputLocation(0, 3), "/[0-9]+/"));
		});
	});

	describe(oneOf.name, () => {
		test("success on second", () => {
			expect(oneOf(
				literal("foo"),
				literal("bar"),
				literal("baz"),
			).parse({
				text: "bar\nremainder",
				location: new InputLocation(0, 0),
			}))
				.toEqual({
					value: "bar",
					remainder: {
						text: "\nremainder",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("success on third", () => {
			expect(oneOf(
				literal("foo"),
				literal("bar"),
				literal("baz"),
			).parse({
				text: "baz\nremainder",
				location: new InputLocation(0, 0),
			}))
				.toEqual({
					value: "baz",
					remainder: {
						text: "\nremainder",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("failure, all matcher fails", () => {
			expect(() => oneOf(
				literal("foo"),
				literal("bar"),
				literal("baz"),
			).parse({
				text: "qux",
				location: new InputLocation(0, 0),
			}))
				.toThrow(new ParseError(new InputLocation(0, 0), "all possibilities failed: expected foo, expected bar, expected baz"));
		});

		test("failure, one of them is a map fail", () => {
			expect(() => oneOf(
				literal("foo"),
				literal("bar").map(_s => {
					throw new Error("Forced failure");
				}),
				literal("baz"),
			).parse({
				text: "bar",
				location: new InputLocation(0, 0),
			}))
				.toThrow(new ParseError(new InputLocation(0, 0), "all possibilities failed: expected foo, Forced failure, expected baz"));
		});
	});

	describe(defer.name, () => {
		test("success", () => {
			const [parser, setParser] = defer<string>();
			setParser(literal("foo"));
			expect(parser.parse({
				text: "foobar",
				location: new InputLocation(0, 0),
			}))
				.toEqual({
					value: "foo",
					remainder: {
						text: "bar",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("failure, use before set", () => {
			const [parser, _setParser] = defer<string>();
			expect(() => parser.parse({
				text: "foobar",
				location: new InputLocation(0, 0),
			}))
				.toThrow("Deferred parser not initialized");
		});

		test("failure, double set", () => {
			const [_parser, setParser] = defer<string>();
			setParser(literal("foo"));
			expect(() => setParser(literal("bar")))
				.toThrow("Deferred parser already initialized");
		});
	});
});
