import XCTest
import experiment

indirect enum Node {
	case Number(Double)
	case Negate(Node)
	case Add(left: Node, right: Node)
	case Subtract(left: Node, right: Node)
	case Multiply(left: Node, right: Node)
	case Divide(left: Node, right: Node)
}

func skipWhitespace<T>(_ p: any Parser<T>) -> any Parser<T> {
	skip(
		maybe(any(string(" "), string("\t"), string("\n"), string("\r"))),
		p
	)
}

func op(_ s: String) -> any Parser<String> {
	skipWhitespace(string(s))
}

func createParser() -> any Parser<Node> {
	let (expression, setExpression): (any Parser<Node>, (any Parser<Node>) -> Void) = deferred()

	/*
	https://www.json.org/json-en.html
	https://stackoverflow.com/a/13340826
	-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?

	ignoring the leading negation because we have an AST node for that
	*/
	let number = regex(/(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/)
		.map { result in
			.success(Node.Number((result as NSString).doubleValue))
		}

	let parenthesis = bracketed(op("("), expression, op(")"))

	let negated = skip(op("-"), expression)

	let term = any(parenthesis, negated, number)

	let multiplyOrDivide = seq2(
		term,
		range(
			seq2(
				any(
					op("*").map { _ in .success(Node.Multiply) },
					op("/").map { _ in .success(Node.Divide) }
				),
				term
			),
			0...
		)
	)
	.map { results in
		let (first, remainder) = results
		return .success(
			remainder.reduce(first) { left, r in
				let (op, right) = r
				return op(left, right)
			})
	}

	let addOrSubtract = seq2(
		multiplyOrDivide,
		range(
			seq2(
				any(
					op("+").map { _ in .success(Node.Add) },
					op("-").map { _ in .success(Node.Subtract) }
				),
				multiplyOrDivide
			),
			0...
		)
	)
	.map { results in
		let (first, remainder) = results
		return .success(
			remainder.reduce(first) { left, r in
				let (op, right) = r
				return op(left, right)
			})
	}

	setExpression(addOrSubtract)

	return expression
}

class CalculatorTest: XCTestCase {
	let parser = createParser()

	func testSuccesses() {
		for (input, expected) in [
			("1", Node.Number(1))
		] {
			let result = parser(input: input)
			XCTAssertEqual(result, .success(expected))
		}
	}
}
