import { describe, expect, it, test } from "bun:test";
import { identifier, label, number } from "./assembler";
import { InputLocation } from "./parser";

describe(__filename, () => {
	describe(number.name, () => {
		it.each([
			["123", 123],
			["0", 0],
			["-123", -123],
			["1.23", 1.23],
			["-1.23", -1.23],
		])("parses %p as %p", (input, expected) => {
			expect(number().parse({
				text: input,
				location: new InputLocation(0, 0),
			})).toEqual({
				value: expected,
				remainder: {
					text: "",
					location: new InputLocation(0, input.length),
				},
			});
		});
	});

	describe(identifier.name, () => {
		it.each([
			["a", "a"],
			["A", "A"],
			["_", "_"],
			["abc", "abc"],
			["ABC", "ABC"],
			["_abc", "_abc"],
			["abc123", "abc123"],
			["_123", "_123"],
			["variable_name", "variable_name"],
			["CamelCase", "CamelCase"],
			["snake_case", "snake_case"],
			["UPPER_CASE", "UPPER_CASE"],
			["a1b2c3", "a1b2c3"],
			["_private", "_private"],
			["__special__", "__special__"],
		])("parses %p as %p", (input, expected) => {
			expect(identifier().parse({
				text: input,
				location: new InputLocation(0, 0),
			})).toEqual({
				value: expected,
				remainder: {
					text: "",
					location: new InputLocation(0, input.length),
				},
			});
		});
	});

	describe(label.name, () => {
		test("success", () => {
			expect(label().parse({
				text: "myLabel:",
				location: new InputLocation(0, 0),
			})).toEqual({
				value: "myLabel",
				remainder: {
					text: "",
					location: new InputLocation(0, "myLabel:".length),
				},
			});
		});
	});
});