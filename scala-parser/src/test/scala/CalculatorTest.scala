import scala.annotation.tailrec
import scala.util.matching.Regex
import scala.util.{Failure, Success, Try}
import parsers.*
import org.scalatest.funsuite.AnyFunSuite
import scala.util.Success
import scala.util.Failure
import org.scalatest.TryValues._
import org.scalatest.TryValues
import org.scalatest.matchers.should.Matchers

enum Node {
	case Number(value: Double) extends Node
	case Negate(value: Node) extends Node
	case Add(left: Node, right: Node) extends Node
	case Subtract(left: Node, right: Node) extends Node
	case Multiply(left: Node, right: Node) extends Node
	case Divide(left: Node, right: Node) extends Node
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
	)

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
	test("TODO do some real tests") {
		val result = expression("-(1 + 2) * 3")
		println(result)
	}
}
