import { anyNumberOf, defer, ignorePrefixAndSuffix, literal, oneOf, padded, Parser, regex, separatedBy, seq } from "./parser";

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

export type Register = "r0" | "r1" | "r2" | "r3" | "r4" | "r5" | "r6" | "r7";

export interface OperandMemory {
	type: "memory";
	value: Register;
}

export interface OperandRegister {
	type: "register";
	value: Register;
}

export interface OperandConstExpr {
	type: "constExpr";
	value: ConstExprAST;
}

export interface OperandLiteral {
	type: "literal";
	value: number;
}

export type UnverifiedOperand = OperandMemory | OperandRegister | OperandConstExpr;

export interface UnverifiedInstruction {
	instruction: string;
	operands: UnverifiedOperand[];
}

export type VerifiedOperand = OperandMemory | OperandRegister | OperandLiteral;

export type VerifiedInstruction =
	| {
		type: "noop";
	}
	| {
		type: "halt";
	}
	| {
		type: "set";
		dest: OperandMemory;
		src: OperandRegister;
	}
	| {
		type: "set";
		dest: OperandRegister;
		src: OperandMemory;
	}
	| {
		type: "set";
		dest: OperandRegister;
		src: OperandLiteral;
	}
	| {
		type: "add";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "sub";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "mul";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "div";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "mod";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "shl";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "shr";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "and";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "or";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "xor";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "jump";
		dest: OperandRegister;
	}
	| {
		type: "jeq";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "jne";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "jlt";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "jle";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "jgt";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "jge";
		dest: OperandRegister;
		left: OperandRegister;
		right: OperandRegister;
	}
	| {
		type: "push";
		src: OperandRegister;
	}
	| {
		type: "pop";
		dest: OperandRegister;
	}
	| {
		type: "fire";
		src: OperandRegister;
	};

export interface Program {
	instructions: VerifiedInstruction[];
}

export function isRegisterName(name: string) {
	return /^r[0-7]$/.test(name);
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

export function register(): Parser<Register> {
	return identifier()
		.filter(isRegisterName)
		.map(name => name as Register);
}

export function memoryAddress(): Parser<Register> {
	return ignorePrefixAndSuffix(
		padded(literal("[")),
		register(),
		padded(literal("]")),
	);
}

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
			padded(literal("(")),
			result,
			padded(literal(")")),
		)
			.map<ConstExprAST>(([_, expr]) => expr),
		numberParser,
		identifierParser,
	);

	const [negatedTerminal, setNegatedTerminal] = defer<ConstExprAST>();
	setNegatedTerminal(oneOf(
		seq(
			padded(literal("-")),
			negatedTerminal
		)
			.map<ConstExprAST>(([_, expr]) => ({ type: "neg", expr })),
		terminal
	));

	const multiplyOrDivide = seq(
		negatedTerminal,
		anyNumberOf(
			seq(
				oneOf(
					padded(literal("*")),
					padded(literal("/")),
					padded(literal("%")),
				),
				negatedTerminal
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

export function operand(): Parser<UnverifiedOperand> {
	return oneOf(
		padded(constExpr())
			.map<UnverifiedOperand>(value => {
				if (value.type === "identifier" && isRegisterName(value.name)) {
					return { type: "register", value: value.name as Register };
				}
				return { type: "constExpr", value };
			}),
		padded(memoryAddress())
			.map<UnverifiedOperand>(value => ({ type: "memory", value })),
		padded(register())
			.map<UnverifiedOperand>(value => ({ type: "register", value })),
	);
}

export function instruction(): Parser<UnverifiedInstruction> {
	return seq(
		padded(identifier()),
		separatedBy(operand(), padded(literal(",")))
	)
		.map<UnverifiedInstruction>(([instruction, operands]) => ({
			instruction,
			operands
		}));
}

// TODO tests
export function assertValidNewConstExprName(constExprMap: Record<string, number>, name: string) {
	if (name in constExprMap) {
		throw new Error(`Duplicate definition: ${name}`);
	}
	if (isRegisterName(name)) {
		throw new Error(`Cannot use register name as const expr: ${name}`);
	}
}

// TODO tests
export function evaluateConstExpr(constExprMap: Record<string, number>, expr: ConstExprAST): number {
	switch (expr.type) {
		case "number":
			return expr.value;
		case "identifier":
			if (expr.name in constExprMap) {
				return constExprMap[expr.name];
			} else {
				throw new Error(`Undefined identifier: ${expr.name}`);
			}
		case "add": {
			const left = evaluateConstExpr(constExprMap, expr.left);
			const right = evaluateConstExpr(constExprMap, expr.right);
			return left + right;
		}
		case "sub": {
			const left = evaluateConstExpr(constExprMap, expr.left);
			const right = evaluateConstExpr(constExprMap, expr.right);
			return left - right;
		}
		case "mul": {
			const left = evaluateConstExpr(constExprMap, expr.left);
			const right = evaluateConstExpr(constExprMap, expr.right);
			return left * right;
		}
		case "div": {
			const left = evaluateConstExpr(constExprMap, expr.left);
			const right = evaluateConstExpr(constExprMap, expr.right);
			return left / right;
		}
		case "mod": {
			const left = evaluateConstExpr(constExprMap, expr.left);
			const right = evaluateConstExpr(constExprMap, expr.right);
			return left % right;
		}
		case "neg": {
			const value = evaluateConstExpr(constExprMap, expr.expr);
			return -value;
		}
	}
}

// TODO tests
export function validateOperand(constExprMap: Record<string, number>, op: UnverifiedOperand): VerifiedOperand {
	if (op.type === "constExpr") {
		const value = evaluateConstExpr(constExprMap, op.value);
		return { type: "literal", value };
	}
	return op;
}

// TODO tests
export function validateInstruction(constExprMap: Record<string, number>, unverifiedInstruction: UnverifiedInstruction): VerifiedInstruction {
	const operands = unverifiedInstruction.operands.map((op) => validateOperand(constExprMap, op));
	switch (unverifiedInstruction.instruction) {
		case "noop":
		case "halt": {
			const expectedOperands = 0;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			return { type: "noop" };
		}
		case "set": {
			const expectedOperands = 2;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "memory" && operands[1].type === "register") {
				return {
					type: "set",
					dest: operands[0],
					src: operands[1],
				};
			}
			if (operands[0].type === "register" && operands[1].type === "memory") {
				return {
					type: "set",
					dest: operands[0],
					src: operands[1],
				};
			}
			if (operands[0].type === "register" && operands[1].type === "literal") {
				return {
					type: "set",
					dest: operands[0],
					src: operands[1],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		case "add":
		case "sub":
		case "mul":
		case "div":
		case "mod":
		case "shl":
		case "shr":
		case "and":
		case "or":
		case "xor": {
			const expectedOperands = 3;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "register" &&
				operands[1].type === "register" &&
				operands[2].type === "register") {
				return {
					type: unverifiedInstruction.instruction,
					dest: operands[0],
					left: operands[1],
					right: operands[2],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		case "jump": {
			const expectedOperands = 1;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "register") {
				return {
					type: "jump",
					dest: operands[0],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		case "jeq":
		case "jne":
		case "jlt":
		case "jle":
		case "jgt":
		case "jge": {
			const expectedOperands = 3;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "register" &&
				operands[1].type === "register" &&
				operands[2].type === "register") {
				return {
					type: unverifiedInstruction.instruction,
					dest: operands[0],
					left: operands[1],
					right: operands[2],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		case "push": {
			const expectedOperands = 1;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "register") {
				return {
					type: "push",
					src: operands[0],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		case "pop": {
			const expectedOperands = 1;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "register") {
				return {
					type: "pop",
					dest: operands[0],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		case "fire": {
			const expectedOperands = 1;
			if (operands.length !== expectedOperands) {
				throw new Error(`Expected ${expectedOperands} operands for ${unverifiedInstruction.instruction}`);
			}
			if (operands[0].type === "register") {
				return {
					type: "fire",
					src: operands[0],
				};
			}
			throw new Error(`Invalid operands for ${unverifiedInstruction.instruction}: ${operands.map(op => op.type).join(", ")}`);
		}
		default:
			throw new Error(`Unknown instruction: ${unverifiedInstruction.instruction}`);
	}
}

// TODO tests
export function program(): Parser<Program> {
	type InstructionOrConstDef =
		| { type: "label"; value: string; }
		| { type: "constDef"; value: ConstDef; }
		| { type: "instruction"; value: UnverifiedInstruction; };

	/*
	just parse the text
	we're not evaluating anything, and we're not proving the instructions are valid
	*/
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

	/*
	now we can go in order and evaluate any expressions, and prove that all instructions are valid
	*/
	const phase2 = phase1
		.map((statements) => {
			const constExprMap: Record<string, number> = {};
			let instructionCount = 0;
			const instructions: VerifiedInstruction[] = [];
			for (const statement of statements) {
				if (statement.type === "label") {
					assertValidNewConstExprName(constExprMap, statement.value);
					constExprMap[statement.value] = instructionCount;
				} else if (statement.type === "constDef") {
					assertValidNewConstExprName(constExprMap, statement.value.name);
					constExprMap[statement.value.name] = evaluateConstExpr(constExprMap, statement.value.value);
				} else if (statement.type === "instruction") {
					instructionCount++;
					instructions.push(validateInstruction(constExprMap, statement.value));
				}
			}
			return {
				instructions,
			};
		});

	return phase2;
}
