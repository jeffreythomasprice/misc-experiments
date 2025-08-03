import { describe, expect, it, test } from "bun:test";
import { constExpr, identifier, isRegisterName, label, memoryAddress, number, register } from "./assembler";
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

	describe(constExpr.name, () => {
		it.each([
			// Simple numbers
			["123", { type: "number", value: 123 } as const],
			["0", { type: "number", value: 0 } as const],
			["-42", { type: "neg", expr: { type: "number", value: 42 } } as const],

			// Simple identifiers
			["foo", { type: "identifier", name: "foo" } as const],
			["MAX_VALUE", { type: "identifier", name: "MAX_VALUE" } as const],

			// Basic arithmetic with order of operations
			["2 + 3 * 4", {
				type: "add",
				left: { type: "number", value: 2 },
				right: {
					type: "mul",
					left: { type: "number", value: 3 },
					right: { type: "number", value: 4 }
				}
			} as const],

			// Multiplication before addition (left associative)
			["5 * 6 + 7 * 8", {
				type: "add",
				left: {
					type: "mul",
					left: { type: "number", value: 5 },
					right: { type: "number", value: 6 }
				},
				right: {
					type: "mul",
					left: { type: "number", value: 7 },
					right: { type: "number", value: 8 }
				}
			} as const],

			// Division and modulo with order of operations
			["10 + 20 / 4 % 3", {
				type: "add",
				left: { type: "number", value: 10 },
				right: {
					type: "mod",
					left: {
						type: "div",
						left: { type: "number", value: 20 },
						right: { type: "number", value: 4 }
					},
					right: { type: "number", value: 3 }
				}
			} as const],

			// Parentheses override order of operations
			["(2 + 3) * 4", {
				type: "mul",
				left: {
					type: "add",
					left: { type: "number", value: 2 },
					right: { type: "number", value: 3 }
				},
				right: { type: "number", value: 4 }
			} as const],

			// Nested parentheses
			["((1 + 2) * 3) + 4", {
				type: "add",
				left: {
					type: "mul",
					left: {
						type: "add",
						left: { type: "number", value: 1 },
						right: { type: "number", value: 2 }
					},
					right: { type: "number", value: 3 }
				},
				right: { type: "number", value: 4 }
			} as const],

			// Complex expression with all operators and precedence
			["10 - 3 * 2 + 8 / 4 % 3", {
				type: "add",
				left: {
					type: "sub",
					left: { type: "number", value: 10 },
					right: {
						type: "mul",
						left: { type: "number", value: 3 },
						right: { type: "number", value: 2 }
					}
				},
				right: {
					type: "mod",
					left: {
						type: "div",
						left: { type: "number", value: 8 },
						right: { type: "number", value: 4 }
					},
					right: { type: "number", value: 3 }
				}
			} as const],

			// Left associativity of same precedence operations
			["100 / 10 / 2", {
				type: "div",
				left: {
					type: "div",
					left: { type: "number", value: 100 },
					right: { type: "number", value: 10 }
				},
				right: { type: "number", value: 2 }
			} as const],

			// Mixed identifiers and numbers
			["x * 2 + y", {
				type: "add",
				left: {
					type: "mul",
					left: { type: "identifier", name: "x" },
					right: { type: "number", value: 2 }
				},
				right: { type: "identifier", name: "y" }
			} as const],

			["-x * 2", {
				type: "mul",
				left: {
					type: "neg",
					expr: { type: "identifier", name: "x" }
				},
				right: { type: "number", value: 2 }
			} as const],

			["-x + 2", {
				type: "add",
				left: {
					type: "neg",
					expr: { type: "identifier", name: "x" }
				},
				right: { type: "number", value: 2 }
			} as const],

			// Unary negation with parentheses
			["-(x + y)", {
				type: "neg",
				expr: {
					type: "add",
					left: { type: "identifier", name: "x" },
					right: { type: "identifier", name: "y" }
				}
			} as const],

			// Multiple unary negations
			["--5", {
				type: "neg",
				expr: {
					type: "neg",
					expr: { type: "number", value: 5 }
				}
			} as const],
		])("parses %p correctly", (input, expected) => {
			expect(constExpr().parse({
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
});