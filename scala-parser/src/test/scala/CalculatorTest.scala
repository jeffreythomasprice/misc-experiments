import scala.annotation.tailrec
import scala.util.matching.Regex
import scala.util.{Failure, Success, Try}
import parsers.*
import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.prop.TableDrivenPropertyChecks._
import scala.util.Success
import scala.util.Failure
import org.scalatest.TryValues.*
import org.scalatest.TryValues
import org.scalatest.matchers.should.Matchers

enum Node {
	case Number(value: Double) extends Node
	case Negate(value: Node) extends Node
	case Add(left: Node, right: Node) extends Node
	case Subtract(left: Node, right: Node) extends Node
	case Multiply(left: Node, right: Node) extends Node
	case Divide(left: Node, right: Node) extends Node

	def eval: Double = this match
		case Node.Number(value) => value
		case Node.Negate(value) => -value.eval
		case Node.Add(left, right) => left.eval + right.eval
		case Node.Subtract(left, right) => left.eval - right.eval
		case Node.Multiply(left, right) => left.eval * right.eval
		case Node.Divide(left, right) => left.eval / right.eval
}

def skipWhitespace[T](p: Parser[T]): Parser[T] =
	skip(
		regex("""^\s*""".r),
		p
	)

val number: Parser[Node] =
	// -?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?
	// https://stackoverflow.com/a/13340826
	skipWhitespace(
		regex("""^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?""".r)
			.map(s =>
				try Success(Node.Number(s.toDouble))
				catch case t: Throwable => Failure(t)
			)
	)

def operator(s: String): Parser[String] =
	skipWhitespace(string(s))

val term: Parser[Node] =
	input =>
		any(
			bracketed(
				operator("("),
				expression,
				operator(")")
			),
			skip(
				operator("-"),
				expression
			)
				.map(x => Success(Node.Negate(x))),
			number
		)(input)

val multiplyOrDivide: Parser[Node] = input =>
	for
		ParseResult(first, remainder) <- term(input)
		ParseResult(more, remainder) <- many0(
			seq2(
				skipWhitespace(
					any(
						operator("*").map(_ => Success((left, right) => Node.Multiply(left, right))),
						operator("/").map(_ => Success((left, right) => Node.Divide(left, right)))
					)
				),
				term
			)
		)(remainder)
	yield ParseResult(
		more.foldLeft(first) {
			(left, opAndRight) =>
				val (op, right) = opAndRight
				op(left, right)
		},
		remainder
	)

val addOrSubtract: Parser[Node] = input =>
	for
		ParseResult(first, remainder) <- multiplyOrDivide(input)
		ParseResult(more, remainder) <- many0(
			seq2(
				skipWhitespace(
					any(
						operator("+").map(_ => Success((left, right) => Node.Add(left, right))),
						operator("-").map(_ => Success((left, right) => Node.Subtract(left, right)))
					)
				),
				multiplyOrDivide
			)
		)(remainder)
	yield ParseResult(
		more.foldLeft(first) {
			(left, opAndRight) =>
				val (op, right) = opAndRight
				op(left, right)
		},
		remainder
	)

val expression: Parser[Node] = input => addOrSubtract(input)

class CalculatorTest extends AnyFunSuite with Matchers with TryValues {
	forAll(Table(
		("input", "expectedResult", "expectedValue"),
		("1", ParseResult(Node.Number(1.0), ""), 1.0),
		("-1.5e2", ParseResult(Node.Negate(Node.Number(150.0)), ""), -150.0),
		(
			"  1 + 2  ",
			ParseResult(
				Node.Add(
					Node.Number(1.0),
					Node.Number(2.0),
				),
				"  "
			),
			3.0
		),
		(
			"-(1+2)*3",
			ParseResult(
				Node.Negate(
					Node.Multiply(
						Node.Add(
							Node.Number(1.0),
							Node.Number(2.0),
						),
						Node.Number(3.0),
					),
				),
				""
			),
			-9.0
		),
	)
	) { (input: String, expectedResult: ParseResult[Node], expectedValue: Double) =>
		val result = expression(input)
		result shouldBe Success(expectedResult)
		result.get.result.eval shouldBe expectedValue
	}

	// TODO failure tests
	forAll(Table(
		("input", "expectedThrowable", "expectedMessage"),
		("-asdf", classOf[ExpectedOneOfException], ""),
	)
	) { (input: String, expectedClass: Class[_], expectedMessage: String) =>
		val result = expression(input)
		result.failure.exception.getClass shouldBe expectedClass
		result.failure.exception.getMessage shouldBe expectedMessage
	}
}
