import { anyNumberOf, defer, literal, oneOf, padded, Parser, regex, separatedBy, seq } from "./parser";

export type ConstExprAST =
	| { type: "number"; value: number }
	| { type: "identifier"; name: string }
	| { type: "add"; left: ConstExprAST; right: ConstExprAST }
	| { type: "sub"; left: ConstExprAST; right: ConstExprAST }
	| { type: "mul"; left: ConstExprAST; right: ConstExprAST }
	| { type: "div"; left: ConstExprAST; right: ConstExprAST }
	| { type: "mod"; left: ConstExprAST; right: ConstExprAST }
	| { type: "neg"; expr: ConstExprAST };

export interface ConstDef {
	name: string;
	value: ConstExprAST;
}

export interface UnverifiedInstruction {
	instruction: string;
	operands: ConstExprAST[];
}

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

// TODO tests
export function constDef(): Parser<ConstDef> {
	return seq(
		padded(identifier()),
		padded(literal("=")),
		constExpr()
	)
		.map<ConstDef>(([name, _, value]) => ({
			name,
			value
		}));
}

// TODO tests
export function instruction(): Parser<UnverifiedInstruction> {
	return seq(
		padded(identifier()),
		separatedBy(constExpr(), padded(literal(",")))
	)
		.map<UnverifiedInstruction>(([instruction, operands]) => ({
			instruction,
			operands
		}));
}

// TODO tests
// TODO return type
export function program() {
	type InstructionOrConstDef =
		| { type: "label"; value: string; }
		| { type: "constDef"; value: ConstDef; }
		| { type: "instruction"; value: UnverifiedInstruction; };

	const phase1 = anyNumberOf(oneOf(
		padded(label())
			.map<InstructionOrConstDef>(value => ({
				type: "label",
				value,
			})),
		constDef()
			.map<InstructionOrConstDef>((value) => ({
				type: "constDef",
				value,
			})),
		instruction()
			.map<InstructionOrConstDef>((value) => ({
				type: "instruction",
				value,
			})),
	));

	// TODO phase 2 should be to validate all labels and const defs, and substitute real values into all instructions, and verify that instructions are all valid
	const phase2 = phase1
		.map((statements) => {
			const constExprMap: Record<string, number> = {};

			const isRegisterName = (name: string) => {
				return /^r[0-7]+$/.test(name);
			};

			const assertValidNewConstExprName = (name: string) => {
				if (name in constExprMap) {
					throw new Error(`Duplicate definition: ${name}`);
				}
				if (isRegisterName(name)) {
					throw new Error(`Cannot use register name as const expr: ${name}`);
				}
			};

			const evaluateConstExpr = (expr: ConstExprAST):
				| { type: "number", value: number; }
				| { type: "register", value: string; } => {
				switch (expr.type) {
					case "number":
						return { type: "number", value: expr.value };
					case "identifier":
						if (expr.name in constExprMap) {
							return { type: "number", value: constExprMap[expr.name] };
						} else {
							throw new Error(`Undefined identifier: ${expr.name}`);
						}
					case "add": {
						const left = evaluateConstExpr(expr.left);
						const right = evaluateConstExpr(expr.right);
						if (left.type === "register" || right.type === "register") {
							throw new Error("Cannot add registers");
						}
						return { type: "number", value: left.value + right.value };
					}
					case "sub": {
						const left = evaluateConstExpr(expr.left);
						const right = evaluateConstExpr(expr.right);
						if (left.type === "register" || right.type === "register") {
							throw new Error("Cannot subtract registers");
						}
						return { type: "number", value: left.value - right.value };
					}
					case "mul": {
						const left = evaluateConstExpr(expr.left);
						const right = evaluateConstExpr(expr.right);
						if (left.type === "register" || right.type === "register") {
							throw new Error("Cannot multiply registers");
						}
						return { type: "number", value: left.value * right.value };
					}
					case "div": {
						const left = evaluateConstExpr(expr.left);
						const right = evaluateConstExpr(expr.right);
						if (left.type === "register" || right.type === "register") {
							throw new Error("Cannot divide registers");
						}
						return { type: "number", value: left.value / right.value };
					}
					case "mod": {
						const left = evaluateConstExpr(expr.left);
						const right = evaluateConstExpr(expr.right);
						if (left.type === "register" || right.type === "register") {
							throw new Error("Cannot modulo registers");
						}
						return { type: "number", value: left.value % right.value };
					}
					case "neg": {
						const value = evaluateConstExpr(expr.expr);
						if (value.type === "register") {
							throw new Error("Cannot negate registers");
						}
						return { type: "number", value: -value.value };
					}
				}
			};

			let instructionCount = 0;
			for (const statement of statements) {
				if (statement.type === "label") {
					assertValidNewConstExprName(statement.value);
					constExprMap[statement.value] = instructionCount;
				} else if (statement.type === "constDef") {
					assertValidNewConstExprName(statement.value.name);
					const result = evaluateConstExpr(statement.value.value);
					if (result.type === "register") {
						throw new Error(`Cannot use register as const expr: ${statement.value.name}`);
					}
					constExprMap[statement.value.name] = result.value;
				} else if (statement.type === "instruction") {
					instructionCount++;

					const operands = statement.value.operands.map(op => evaluateConstExpr(op));

					// TODO validate instruction
				}
			}
		});
}
