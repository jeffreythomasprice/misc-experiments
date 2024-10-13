import { describe, expect, it } from "bun:test";
import { any, charRange, defer, delimited, many0, many1, optional, preceded, seq, string, type Parser, type Result } from "./parser";

interface Node {
	toJSON(): object;
	eval(): number;
}

class NumberNode implements Node {
	constructor(readonly value: number) { }

	toJSON(): object {
		return { number: this.value };
	}

	eval(): number {
		return this.value;
	}
}

class NegateNode implements Node {
	constructor(readonly value: Node) { }

	toJSON(): object {
		return { negate: this.value.toJSON() };
	}

	eval(): number {
		return -this.value.eval();
	}
}

class AddNode implements Node {
	constructor(
		readonly left: Node,
		readonly right: Node,
	) { }

	toJSON(): object {
		return {
			add: {
				left: this.left.toJSON(),
				right: this.right.toJSON(),
			},
		};
	}

	eval(): number {
		return this.left.eval() + this.right.eval();
	}
}

class SubtractNode implements Node {
	constructor(
		readonly left: Node,
		readonly right: Node,
	) { }

	toJSON(): object {
		return {
			subtract: {
				left: this.left.toJSON(),
				right: this.right.toJSON(),
			},
		};
	}

	eval(): number {
		return this.left.eval() - this.right.eval();
	}
}

class MultiplyNode implements Node {
	constructor(
		readonly left: Node,
		readonly right: Node,
	) { }

	toJSON(): object {
		return {
			multiply: {
				left: this.left.toJSON(),
				right: this.right.toJSON(),
			},
		};
	}

	eval(): number {
		return this.left.eval() * this.right.eval();
	}
}

class DivideNode implements Node {
	constructor(
		readonly left: Node,
		readonly right: Node,
	) { }

	toJSON(): object {
		return {
			divide: {
				left: this.left.toJSON(),
				right: this.right.toJSON(),
			},
		};
	}

	eval(): number {
		return this.left.eval() / this.right.eval();
	}
}

let expressionParser: Parser<Node> | null = null;

function parse(input: string): Result<Node> {
	expressionParser ??= createExpressionParser();
	return expressionParser(input);
}

function createExpressionParser(): Parser<Node> {
	const [setter, result] = defer<Node>();
	setter(createAddOrSubrtact(result));
	return result;
}

function createAddOrSubrtact(expression: Parser<Node>): Parser<Node> {
	const nextParser = createMultiplyOrDivide(expression);
	return seq(
		nextParser,
		many0(
			seq(
				any(
					skipWhitespace(string("+"))
						.map(_ => AddNode),
					skipWhitespace(string("-"))
						.map(_ => SubtractNode),
				),
				nextParser,
			),
		),
	)
		.map(([first, rest]) =>
			rest.reduce(
				(left, [op, right]) => new op(left, right),
				first,
			)
		);
}

function createMultiplyOrDivide(expression: Parser<Node>): Parser<Node> {
	const nextParser = createTerminal(expression);
	return seq(
		nextParser,
		many0(
			seq(
				any(
					skipWhitespace(string("*"))
						.map(_ => MultiplyNode),
					skipWhitespace(string("/"))
						.map(_ => DivideNode),
				),
				nextParser,
			),
		),
	)
		.map(([first, rest]) =>
			rest.reduce(
				(left, [op, right]) => new op(left, right),
				first,
			)
		);
}

function createTerminal(expression: Parser<Node>): Parser<Node> {
	return any(
		preceded(
			skipWhitespace(string("-")),
			expression,
		)
			.map(x => new NegateNode(x)),
		delimited(
			skipWhitespace(string("(")),
			expression,
			skipWhitespace(string(")")),
		),
		skipWhitespace(createNumberParser()),
	);
}

function skipWhitespace<T>(p: Parser<T>): Parser<T> {
	return preceded(
		many0(any(
			string(" "),
			string("\t"),
			string("\n"),
			string("\r"),
		)),
		p,
	);
}

function createNumberParser(): Parser<Node> {
	/*
	https://www.crockford.com/mckeeman.html

	slightly translated to a more comprehensible notation

	number
		integer fraction? exponent?
	
	integer
		| digit
		| onenine digits
		| "-" digit
		| "-" onenine digits

	digits
		digit+

	digit
		"0" .. "9"

	onenine
		"1" .. "9"

	fraction
		| "." digits

	exponent
		| "E" sign? digits
		| "e" sign? digits

	sign
		| "+"
		| "-"
	*/

	const digit = charRange("0", "9");
	const oneNine = charRange("1", "9");
	const digits = many1(digit)
		.map(x => x.join(""));
	const fraction = seq(
		string("."),
		digits,
	)
		.map(x => x.join(""));
	const exponent = seq(
		any(string("e"), string("E")),
		optional(any(string("+"), string("-"))),
		digits,
	)
		.map(x => x.join(""));
	const integer = seq(
		optional(string("-"))
			.map(x => x ?? ""),
		any(
			seq(
				oneNine,
				digits,
			)
				.map(x => x.join("")),
			digit,
		),
	)
		.map(x => x.join(""));
	return seq(
		integer,
		optional(fraction)
			.map(x => x ?? ""),
		optional(exponent)
			.map(x => x ?? ""),
	)
		.map(x => x.join(""))
		.map(x => parseFloat(x))
		.map(x => new NumberNode(x));
}

describe(__filename, () => {
	it.each([
		[
			"1",
			{
				success: true,
				result: new NumberNode(1),
				remainder: "",
			},
			1,
		],
		[
			"  \t123    ",
			{
				success: true,
				result: new NumberNode(123),
				remainder: "    ",
			},
			123,
		],
		[
			"1.5",
			{
				success: true,
				result: new NumberNode(1.5),
				remainder: "",
			},
			1.5,
		],
		[
			" 2e5",
			{
				success: true,
				result: new NumberNode(2e5),
				remainder: "",
			},
			2e5,
		],
		[
			" 1E-2",
			{
				success: true,
				result: new NumberNode(1e-2),
				remainder: "",
			},
			1e-2,
		],
		[
			"-(1+2)*3/4",
			{
				success: true,
				result: new NegateNode(
					new DivideNode(
						new MultiplyNode(
							new AddNode(
								new NumberNode(1),
								new NumberNode(2),
							),
							new NumberNode(3),
						),
						new NumberNode(4),
					),
				),
				remainder: "",
			},
			-2.25,
		],
	])("success %# input=\"%s\"", (input, expectedNode, expectedValue) => {
		const result = parse(input);
		expect(result as any).toEqual(expectedNode);
		if (result.success) {
			expect(result.result.eval()).toEqual(expectedValue);
		}
	});
});