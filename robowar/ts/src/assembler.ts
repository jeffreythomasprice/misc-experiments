import { anyNumberOf, defer, literal, oneOf, padded, Parser, regex, seq } from "./parser";

export type ConstExprAST =
	| { type: "number"; value: number }
	| { type: "identifier"; name: string }
	| { type: "add"; left: ConstExprAST; right: ConstExprAST }
	| { type: "sub"; left: ConstExprAST; right: ConstExprAST }
	| { type: "mul"; left: ConstExprAST; right: ConstExprAST }
	| { type: "div"; left: ConstExprAST; right: ConstExprAST }
	| { type: "mod"; left: ConstExprAST; right: ConstExprAST }
	| { type: "neg"; expr: ConstExprAST };

export function number(): Parser<number> {
	return regex(/^-?(?:\d+\.?\d*|\.\d+)/)
		.map(str => parseFloat(str));
}

export function identifier(): Parser<string> {
	return regex(/^[a-zA-Z_][a-zA-Z0-9_]*/)
}

export function label(): Parser<string> {
	return seq(
		identifier(),
		literal(":"),
	)
		.map(([name]) => name);
}

// TODO tests
export function constExpr(): Parser<ConstExprAST> {
	const [result, setResult] = defer<ConstExprAST>();

	const numberParser = padded(
		number()
			.map<ConstExprAST>(value => ({ type: "number", value }))
	);

	const identifierParser = padded(
		identifier()
			.map<ConstExprAST>(name => ({ type: "identifier", name }))
	);

	const terminal = oneOf(
		seq(
			padded(literal("-")),
			result,
		)
			.map<ConstExprAST>(([_, expr]) => ({ type: "neg", expr })),
		seq(
			padded(literal("(")),
			result,
			padded(literal(")")),
		)
			.map<ConstExprAST>(([_, expr]) => expr),
		numberParser,
		identifierParser,
	);

	const multiplyOrDivide = seq(
		terminal,
		anyNumberOf(
			seq(
				oneOf(
					padded(literal("*")),
					padded(literal("/")),
					padded(literal("%")),
				),
				terminal
			)
		)
	)
		.map<ConstExprAST>(([first, rest]) => {
			let result: ConstExprAST = first
			for (const [op, next] of rest) {
				switch (op) {
					case "*":
						result = { type: "mul", left: result, right: next };
						break;
					case "/":
						result = { type: "div", left: result, right: next };
						break;
					case "%":
						result = { type: "mod", left: result, right: next };
						break;
				}
			}
			return result;
		});

	const addOrSubtract = seq(
		multiplyOrDivide,
		anyNumberOf(
			seq(
				oneOf(
					padded(literal("+")),
					padded(literal("-")),
				),
				multiplyOrDivide
			)
		)
	)
		.map<ConstExprAST>(([first, rest]) => {
			let result: ConstExprAST = first;
			for (const [op, next] of rest) {
				switch (op) {
					case "+":
						result = { type: "add", left: result, right: next };
						break;
					case "-":
						result = { type: "sub", left: result, right: next };
						break;
				}
			}
			return result;
		});

	setResult(addOrSubtract);

	return result;
}