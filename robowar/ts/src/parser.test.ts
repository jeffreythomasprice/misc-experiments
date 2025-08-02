import { describe, expect, test } from "bun:test";
import { InputLocation, literal, oneOf, ExpectedError, regex, seq, ParseError, defer, anyNumberOf, optional, ignorePrefix as ignorePrefix, ignoreSuffix, ignorePrefixAndSuffix, padded } from "./parser";

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

	describe(anyNumberOf.name, () => {
		test("zero matches", () => {
			expect(anyNumberOf(literal("foo"))
				.parse({
					text: "bar",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: [],
					remainder: {
						text: "bar",
						location: new InputLocation(0, 0),
					},
				});
		});

		test("one match", () => {
			expect(anyNumberOf(literal("foo"))
				.parse({
					text: "foobar",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: ["foo"],
					remainder: {
						text: "bar",
						location: new InputLocation(0, 3),
					},
				});
		});

		test("two matches", () => {
			expect(anyNumberOf(literal("foo"))
				.parse({
					text: "foofoobar",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: ["foo", "foo"],
					remainder: {
						text: "bar",
						location: new InputLocation(0, 6),
					},
				});
		});

		test("rethrows unexpected errors", () => {
			expect(() => anyNumberOf(literal("foo").map(_s => {
				throw new Error("Unexpected error");
			})).parse({
				text: "foo",
				location: new InputLocation(0, 0),
			}))
				.toThrow("Unexpected error");
		});

		test("empty string input", () => {
			expect(anyNumberOf(literal("foo"))
				.parse({
					text: "",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: [],
					remainder: {
						text: "",
						location: new InputLocation(0, 0),
					},
				});
		});
	});

	describe(optional.name, () => {
		test("success, returns parsed value", () => {
			expect(optional(literal("foo"))
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

		test("failure, returns null and doesn't advance", () => {
			expect(optional(literal("foo"))
				.parse({
					text: "bar",
					location: new InputLocation(1, 5),
				}))
				.toEqual({
					value: null,
					remainder: {
						text: "bar",
						location: new InputLocation(1, 5),
					},
				});
		});

		test("rethrows unexpected errors from map function", () => {
			expect(() => optional(literal("foo").map(_s => {
				throw new Error("Unexpected error in map");
			})).parse({
				text: "foo",
				location: new InputLocation(0, 0),
			}))
				.toThrow("Unexpected error in map");
		});

		test("empty string input", () => {
			expect(optional(literal("foo"))
				.parse({
					text: "",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: null,
					remainder: {
						text: "",
						location: new InputLocation(0, 0),
					},
				});
		});

		test("success at end of input", () => {
			expect(optional(literal("foo"))
				.parse({
					text: "foo",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "foo",
					remainder: {
						text: "",
						location: new InputLocation(0, 3),
					},
				});
		});
	});

	describe(ignorePrefix.name, () => {
		test("with prefix present", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");

			expect(ignorePrefix(prefixParser, mainParser)
				.parse({
					text: "prefixmain",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 10),
					},
				});
		});

		test("without prefix present", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");

			expect(ignorePrefix(prefixParser, mainParser)
				.parse({
					text: "main",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 4),
					},
				});
		});

		test("failure when main parser fails", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");

			expect(() => ignorePrefix(prefixParser, mainParser)
				.parse({
					text: "prefixfoo",
					location: new InputLocation(0, 0),
				}))
				.toThrow(new ExpectedError(new InputLocation(0, 6), "main"));
		});
	});

	describe(ignoreSuffix.name, () => {
		test("with suffix present", () => {
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(ignoreSuffix(mainParser, suffixParser)
				.parse({
					text: "mainsuffix",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 10),
					},
				});
		});

		test("without suffix present", () => {
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(ignoreSuffix(mainParser, suffixParser)
				.parse({
					text: "main",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 4),
					},
				});
		});

		test("failure when main parser fails", () => {
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(() => ignoreSuffix(mainParser, suffixParser)
				.parse({
					text: "foo",
					location: new InputLocation(0, 0),
				}))
				.toThrow(new ExpectedError(new InputLocation(0, 0), "main"));
		});
	});

	describe(ignorePrefixAndSuffix.name, () => {
		test("with both prefix and suffix present", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(ignorePrefixAndSuffix(prefixParser, mainParser, suffixParser)
				.parse({
					text: "prefixmainsuffix",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 16),
					},
				});
		});

		test("with only prefix present", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(ignorePrefixAndSuffix(prefixParser, mainParser, suffixParser)
				.parse({
					text: "prefixmain",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 10),
					},
				});
		});

		test("with only suffix present", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(ignorePrefixAndSuffix(prefixParser, mainParser, suffixParser)
				.parse({
					text: "mainsuffix",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 10),
					},
				});
		});

		test("with neither prefix nor suffix present", () => {
			const prefixParser = literal("prefix");
			const mainParser = literal("main");
			const suffixParser = literal("suffix");

			expect(ignorePrefixAndSuffix(prefixParser, mainParser, suffixParser)
				.parse({
					text: "main",
					location: new InputLocation(0, 0),
				}))
				.toEqual({
					value: "main",
					remainder: {
						text: "",
						location: new InputLocation(0, 4),
					},
				});
		});
	});

	describe(padded.name, () => {
		test("ignore whitespace", () => {
			const parser = padded(literal("foo"));
			expect(parser.parse({
				text: "   foo   ",
				location: new InputLocation(0, 0),
			})).toEqual({
				value: "foo",
				remainder: {
					text: "",
					location: new InputLocation(0, 9),
				},
			});
		});
	});
});
