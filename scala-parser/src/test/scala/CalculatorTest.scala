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
import parsers.*

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
    ),
    number
  )

val multiplyOrDivide: Parser[Node] = input => ???

val addOrSubtract: Parser[Node] = input => ???

val expression: Parser[Node] = input => addOrSubtract(input)

class CalculatorTest extends AnyFunSuite with Matchers with TryValues {
  test("TODO do some real tests") {
    val result = expression("1")
  }
}
