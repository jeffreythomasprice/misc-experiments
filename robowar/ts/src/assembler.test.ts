import { describe, expect, it, test } from "bun:test";
import { identifier, isRegisterName, label, memoryAddress, number, register } from "./assembler";
import { InputLocation } from "./parser";

describe(__filename, () => {
	describe(isRegisterName.name, () => {
		it.each([
			// Valid register names
			["r0", true],
			["r1", true],
			["r2", true],
			["r3", true],
			["r4", true],
			["r5", true],
			["r6", true],
			["r7", true],
			// Invalid register names
			["r8", false],
			["r9", false],
			["r10", false],
			["r-1", false],
			["ra", false],
			["R0", false],
			["r", false],
			["0", false],
			["", false],
			["r0r1", false],
			["r00", false],
			["r01", false],
			["r123", false],
			["register", false],
			[" r0", false],
			["r0 ", false],
			["x0", false],
			["s0", false],
		])("input %s, returns %p", (input, expected) => {
			expect(isRegisterName(input)).toBe(expected);
		});
	});

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

	describe(register.name, () => {
		test("success", () => {
			expect(register().parse({
				text: "r0",
				location: new InputLocation(0, 0),
			})).toEqual({
				value: "r0",
				remainder: {
					text: "",
					location: new InputLocation(0, "r0".length),
				},
			});
		});

		test("failure", () => {
			expect(() => register().parse({
				text: "r8",
				location: new InputLocation(0, 0),
			})).toThrow(new Error("Value didn't pass filter"));
		});
	});

	describe(memoryAddress.name, () => {
		test("success", () => {
			expect(memoryAddress().parse({
				text: "[r0]",
				location: new InputLocation(0, 0),
			})).toEqual({
				value: "r0",
				remainder: {
					text: "",
					location: new InputLocation(0, "[r0]".length),
				},
			});
		});

		test("failure", () => {
			expect(() => memoryAddress().parse({
				text: "[r8]",
				location: new InputLocation(0, 0),
			})).toThrow(new Error("Value didn't pass filter"));
		});
	});
});