import { describe, expect, it } from "bun:test";
import { any, charRange, count, defer, delimited, EndOfInput, ExpectedError, many0, many1, MultiError, optional, preceded, seq, string, terminated } from "./parser";

describe(__filename, () => {
	describe(string.name, () => {
		it("success", () => {
			expect(string("foo")("foobar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});

		it("failure", () => {
			expect(string("foo")("barfoo"))
				.toEqual({
					success: false,
					reason: new ExpectedError("foo", "bar"),
				});
		});

		it("failure, end of input", () => {
			expect(string("foo")("fo"))
				.toEqual({
					success: false,
					reason: new EndOfInput("foo"),
				});
		});
	});

	describe("map", () => {
		it("success", () => {
			expect(string("foo").map(x => x.length)("foobar"))
				.toEqual({
					success: true,
					result: 3,
					remainder: "bar",
				});
		});

		it("failure", () => {
			expect(string("foo").map(x => x.length)("barfoo"))
				.toEqual({
					success: false,
					reason: new ExpectedError("foo", "bar"),
				});
		});
	});

	describe(charRange.name, () => {
		it("success", () => {
			expect(charRange("0", "9")("123"))
				.toEqual({
					success: true,
					result: "1",
					remainder: "23",
				});
		});

		it("failure, bad next char", () => {
			expect(charRange("0", "9")("abc"))
				.toEqual({
					success: false,
					reason: new ExpectedError("0..9", "a"),
				});
		});

		it("failure, end of input", () => {
			expect(charRange("0", "9")(""))
				.toEqual({
					success: false,
					reason: new EndOfInput("0..9"),
				});
		});

		it("bad constructor args", () => {
			expect(() => charRange("01", "9")(""))
				.toThrow(new Error("arguments should be single characters, got 01..9"));
			expect(() => charRange("0", "99")(""))
				.toThrow(new Error("arguments should be single characters, got 0..99"));
		});
	});

	describe(seq.name, () => {
		it("success", () => {
			expect(seq(
				charRange('0', '9'),
				charRange('a', 'z'),
				string("!!!"),
			)("1g!!!__"))
				.toEqual({
					success: true,
					result: ["1", "g", "!!!"],
					remainder: "__",
				});
		});

		it("success with mapper", () => {
			const digit = charRange('0', '9')
				.map(x => parseInt(x, 10));
			expect(seq(
				digit,
				digit,
			)
				.map(([a, b]) => a + b)
				("56__"))
				.toEqual({
					success: true,
					result: 11,
					remainder: "__",
				});
		});

		it("failure", () => {
			expect(seq(
				charRange('0', '9'),
				charRange('a', 'z'),
			)("1_"))
				.toEqual({
					success: false,
					reason: new ExpectedError("a..z", "_"),
				});
			expect(seq(
				charRange('0', '9'),
				charRange('a', 'z'),
			)("_a"))
				.toEqual({
					success: false,
					reason: new ExpectedError("0..9", "_"),
				});
		});
	});

	describe(any.name, () => {
		it("success on first", () => {
			expect(any(
				string("foo"),
				string("bar"),
			)("foobar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});

		it("success on second", () => {
			expect(any(
				string("bar"),
				string("foo"),
			)("foobar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});

		it("neither", () => {
			expect(any(
				string("foo"),
				string("bar"),
			)("baz"))
				.toEqual({
					success: false,
					reason: new MultiError([
						new ExpectedError("foo", "baz"),
						new ExpectedError("bar", "baz"),
					]),
				});
		});
	});

	describe(count.name, () => {
		it("success", () => {
			expect(count(string("foo"), 3)("foofoofoofoobar"))
				.toEqual({
					success: true,
					result: [
						"foo",
						"foo",
						"foo",
					],
					remainder: "foobar",
				});
		});

		it("failure", () => {
			expect(count(string("foo"), 3)("foofoo"))
				.toEqual({
					success: false,
					reason: new EndOfInput("foo"),
				});
		});

		it("bad inputs", () => {
			expect(() => count(string("foo"), 1.5))
				.toThrow(new Error("must provide a positive integer for count, got 1.5"));
			expect(() => count(string("foo"), 0))
				.toThrow(new Error("must provide a positive integer for count, got 0"));
			expect(() => count(string("foo"), -1))
				.toThrow(new Error("must provide a positive integer for count, got -1"));
		});
	});

	describe(many0.name, () => {
		it("success, none", () => {
			expect(many0(string("foo"))("bar"))
				.toEqual({
					success: true,
					result: [],
					remainder: "bar",
				});
		});

		it("success, one", () => {
			expect(many0(string("foo"))("foobar"))
				.toEqual({
					success: true,
					result: [
						"foo",
					],
					remainder: "bar",
				});
		});

		it("success, two", () => {
			expect(many0(string("foo"))("foofoobar"))
				.toEqual({
					success: true,
					result: [
						"foo",
						"foo",
					],
					remainder: "bar",
				});
		});
	});

	describe(many1.name, () => {
		it("failure, none", () => {
			expect(many1(string("foo"))("bar"))
				.toEqual({
					success: false,
					reason: new ExpectedError("foo", "bar"),
				});
		});

		it("success, one", () => {
			expect(many1(string("foo"))("foobar"))
				.toEqual({
					success: true,
					result: [
						"foo",
					],
					remainder: "bar",
				});
		});

		it("success, two", () => {
			expect(many1(string("foo"))("foofoobar"))
				.toEqual({
					success: true,
					result: [
						"foo",
						"foo",
					],
					remainder: "bar",
				});
		});
	});

	describe(optional.name, () => {
		it("success, some", () => {
			expect(optional(string("foo"))("foobar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});

		it("success, none", () => {
			expect(optional(string("foo"))("baz"))
				.toEqual({
					success: true,
					result: null,
					remainder: "baz",
				});
		});
	});

	describe(delimited.name, () => {
		it("success", () => {
			expect(delimited(
				string("("),
				string("foo"),
				string(")"),
			)("(foo)bar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});
	});

	describe(preceded.name, () => {
		it("success", () => {
			expect(preceded(
				string("__"),
				string("foo"),
			)("__foobar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});
	});

	describe(terminated.name, () => {
		it("success", () => {
			expect(terminated(
				string("foo"),
				string("__"),
			)("foo__bar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});
	});

	describe(defer.name, () => {
		it("success", () => {
			const [set, p] = defer<string>();
			set(string("foo"));
			expect(p("foobar"))
				.toEqual({
					success: true,
					result: "foo",
					remainder: "bar",
				});
		});

		it("failure, use before init", () => {
			const [_set, p] = defer<string>();
			expect(() => p("foo"))
				.toThrow("deferred parser used before it was initialized");
		});
	});
});