import java.io.BufferedReader
import java.io.StringReader
import scala.util.matching.Regex

enum TokenType:
    case Whitespace
    case Number
    case Plus
    case Minus
    case Times
    case Divide
    case LeftParenthesis
    case RightParenthesis

trait Node:
    def eval(): Double

class NumberNode(value: Double) extends Node:
    override def eval(): Double = value

class NegateNode(child: Node) extends Node:
    override def eval(): Double = -child.eval()

private object Init {
    val tokenizer =
        TokenizerBuilder()
            .regex(TokenType.Whitespace, Regex("[ \t\r\n]+"))
            .regex(TokenType.Number, Regex("[0-9]+"))
            .literal(TokenType.Plus, "+")
            .literal(TokenType.Minus, "-")
            .literal(TokenType.Times, "*")
            .literal(TokenType.Divide, "/")
            .literal(TokenType.LeftParenthesis, "(")
            .literal(TokenType.RightParenthesis, ")")
            .build()

    val expression = Deferred[TokenType, Node]()

    val number =
        SingleToken(TokenType.Number).map(token =>
            NumberNode(token.value.toInt).asInstanceOf[Node]
        )

    val negate = Sequence2(
      SingleToken(TokenType.Minus),
      expression
    ).map(result => NegateNode(result._2).asInstanceOf[Node])

    val subexpression = Sequence3(
      SingleToken(TokenType.LeftParenthesis),
      expression,
      SingleToken(TokenType.RightParenthesis)
    ).map(_._2)

    val numberOrNegateOrSubexpression = OneOf(number, negate, subexpression)

    val multiplyOrDivide = Sequence2(
      numberOrNegateOrSubexpression,
      Sequence2(
        OneOf(
          SingleToken(TokenType.Times).map(_ =>
              def f(left: Node, right: Node): Node = ???
              f
          ),
          SingleToken(TokenType.Divide).map(_ =>
              def f(left: Node, right: Node): Node = ???
              f
          )
        ),
        numberOrNegateOrSubexpression
      ).atLeast(0)
    ).map(nodes =>
        val (first, remainder) = nodes
        var result = first
        remainder.foreach(x =>
            val (op, right) = x
            result = op(result, right)
        )
        result
    )

    val addOrSubtract = Sequence2(
      multiplyOrDivide,
      Sequence2(
        OneOf(
          SingleToken(TokenType.Times).map(_ =>
              def f(left: Node, right: Node): Node = ???
              f
          ),
          SingleToken(TokenType.Divide).map(_ =>
              def f(left: Node, right: Node): Node = ???
              f
          )
        ),
        multiplyOrDivide
      ).atLeast(0)
    ).map(nodes =>
        val (first, remainder) = nodes
        var result = first
        remainder.foreach(x =>
            val (op, right) = x
            result = op(result, right)
        )
        result
    )

    expression.resolve(addOrSubtract)

    @main
    def main =
        val input = BufferedReader(StringReader("1 + 2"))
        val tokens = BufferedStream(
          IterableStream(
            tokenizer
                .apply(input)
                .iterator
                .filter(_.name != TokenType.Whitespace)
          )
        )
        val result = expression.apply(tokens)

        tokens.next() match
            case None        => ()
            case Some(value) => throw Exception(s"unexpected token $value")

        input.peek(1) match
            case None        => ()
            case Some(value) =>
                // TODO should have position after last token
                throw Exception(s"unexpected character $value")

        println(s"parse result = $result")
        println(s"eval result = ${result.get.eval()}")
}
