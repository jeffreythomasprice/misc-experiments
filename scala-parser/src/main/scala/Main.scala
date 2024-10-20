import scala.annotation.tailrec
import scala.util.matching.Regex
import scala.util.{Failure, Success, Try}
import parsers.*

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

@main def main(): Unit =
  println(expression("-1.5e2 foobar"))
