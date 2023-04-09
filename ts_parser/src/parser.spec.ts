import { filter } from "rxjs";

import { Tokenizer } from "./tokenizer";
import { anyNumberOf, defer, literal, oneOf, parse, repeat, sequence } from "./parser";

enum TokenName {
	Number = "NUMBER",
	Whitespace = "WHITESPACE",
	Plus = "PLUS",
	Minus = "MINUS",
	Times = "TIMES",
	Divide = "DIVIDE",
	LeftParenthesis = "LEFT_PARENTHESIS",
	RightParenthesis = "RIGHT_PARENTHESIS",
}

interface Node {
	eval(): number;
}

class NumberNode implements Node {
	constructor(private readonly value: number) { }

	toString(): string {
		return this.value.toString();
	}

	eval(): number {
		return this.value;
	}
}

class UnaryOperationNode implements Node {
	constructor(
		private readonly node: Node,
		private readonly f: (value: number) => number,
		private readonly name: string,
	) { }

	toString(): string {
		return `${this.name}(${this.node})`;
	}

	eval(): number {
		return this.f(this.node.eval());
	}
}

class BinaryOperationNode implements Node {
	constructor(
		private readonly left: Node,
		private readonly right: Node,
		private readonly f: (left: number, right: number) => number,
		private readonly name: string,
	) { }

	toString(): string {
		return `${this.name}(${this.left}, ${this.right})`;
	}

	eval(): number {
		return this.f(this.left.eval(), this.right.eval());
	}
}

describe("parser", () => {
	it("TODO JEFF testing", async () => {
		const tokenizer = new Tokenizer.Builder()
			.add(TokenName.Number, /[0-9]+/)
			.add(TokenName.Whitespace, /\s+/)
			.add(TokenName.Plus, "+")
			.add(TokenName.Minus, "-")
			.add(TokenName.Times, "*")
			.add(TokenName.Divide, "/")
			.add(TokenName.LeftParenthesis, "(")
			.add(TokenName.RightParenthesis, ")")
			.build();

		const number = literal(TokenName.Number)
			.map((_, [token]): Node => {
				return new NumberNode(Number.parseInt(token.value, 10));
			});

		const [fullExpression, setFullExpression] = defer<TokenName, Node>();

		const term = oneOf(
			number,
			sequence(
				literal(TokenName.LeftParenthesis),
				fullExpression,
				literal(TokenName.RightParenthesis),
			)
				.map(([, result]) => result),
			sequence(
				literal(TokenName.Minus),
				fullExpression
			)
				.map(([, result]) => new UnaryOperationNode(
					result,
					(value) => -value,
					"negate"
				))
		);

		const mulop = sequence(
			term,
			anyNumberOf(
				sequence(
					oneOf(
						literal(TokenName.Times)
							.map((): ((left: Node, right: Node) => BinaryOperationNode) => {
								return (left, right) => {
									return new BinaryOperationNode(
										left,
										right,
										(left, right) => left * right,
										"multiply"
									);
								};
							}),
						literal(TokenName.Divide)
							.map((): ((left: Node, right: Node) => BinaryOperationNode) => {
								return (left, right) => {
									return new BinaryOperationNode(
										left,
										right,
										(left, right) => left / right,
										"divide"
									);
								};
							}),

					),
					term
				)
			),
		)
			.map(([first, remainder]) => remainder.reduce(
				(left, [opFactory, right]) => opFactory(left, right),
				first
			));

		const addop = sequence(
			mulop,
			anyNumberOf(
				sequence(
					oneOf(
						literal(TokenName.Plus)
							.map((): ((left: Node, right: Node) => BinaryOperationNode) => {
								return (left, right) => {
									return new BinaryOperationNode(
										left,
										right,
										(left, right) => left + right,
										"add"
									);
								};
							}),
						literal(TokenName.Minus)
							.map((): ((left: Node, right: Node) => BinaryOperationNode) => {
								return (left, right) => {
									return new BinaryOperationNode(
										left,
										right,
										(left, right) => left - right,
										"subtract"
									);
								};
							}),
					),
					mulop
				)
			),
		)
			.map(([first, remainder]) => remainder.reduce(
				(left, [opFactory, right]) => opFactory(left, right),
				first
			));

		setFullExpression(addop);

		const results = await parse(
			tokenizer.tokenize("(1 + 2)/3*-4")
				.pipe(filter((token) => token.name !== TokenName.Whitespace)),
			fullExpression
		);
		if (results) {
			console.log("TODO JEFF parse result", results.result.toString());
		} else {
			throw new Error("failed to parse");
		}
	});
});
