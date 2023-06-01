import java.io.BufferedReader
import java.util.Collection
import scala.collection.mutable.ListBuffer
trait Parser[Name, Result]:
    def apply(input: BufferedStream[Token[Name]]): Option[Result]

class SingleToken[Name](expected: Name) extends Parser[Name, Token[Name]]:
    override def apply(
        input: BufferedStream[Token[Name]]
    ): Option[Token[Name]] =
        input.peek() match
            case None => None
            case Some(result) =>
                if result.name == expected then
                    input.next()
                    Some(result)
                else None

class Sequence2[Name, Result1, Result2](
    parser1: Parser[Name, Result1],
    parser2: Parser[Name, Result2]
) extends Parser[Name, (Result1, Result2)]:
    override def apply(
        input: BufferedStream[Token[Name]]
    ): Option[(Result1, Result2)] =
        val saved = input.position
        parser1
            .apply(input)
            .flatMap(result1 =>
                parser2.apply(input).map(result2 => (result1, result2))
            )
            .ifEmpty(() => input.position = saved)

class Sequence3[Name, Result1, Result2, Result3](
    parser1: Parser[Name, Result1],
    parser2: Parser[Name, Result2],
    parser3: Parser[Name, Result3]
) extends Parser[Name, (Result1, Result2, Result3)]:
    override def apply(
        input: BufferedStream[Token[Name]]
    ): Option[(Result1, Result2, Result3)] =
        val saved = input.position
        parser1
            .apply(input)
            .flatMap(result1 =>
                parser2.apply(input).map(result2 => (result1, result2))
            )
            .flatMap(results =>
                parser3
                    .apply(input)
                    .map(result3 => (results._1, results._2, result3))
            )
            .ifEmpty(() => input.position = saved)

class Sequence4[Name, Result1, Result2, Result3, Result4](
    parser1: Parser[Name, Result1],
    parser2: Parser[Name, Result2],
    parser3: Parser[Name, Result3],
    parser4: Parser[Name, Result4]
) extends Parser[Name, (Result1, Result2, Result3, Result4)]:
    override def apply(
        input: BufferedStream[Token[Name]]
    ): Option[(Result1, Result2, Result3, Result4)] =
        val saved = input.position
        parser1
            .apply(input)
            .flatMap(result1 =>
                parser2.apply(input).map(result2 => (result1, result2))
            )
            .flatMap(results =>
                parser3
                    .apply(input)
                    .map(result3 => (results._1, results._2, result3))
            )
            .flatMap(results =>
                parser4
                    .apply(input)
                    .map(result4 =>
                        (results._1, results._2, results._3, result4)
                    )
            )
            .ifEmpty(() => input.position = saved)

class OneOf[Name, Result](parsers: Iterable[Parser[Name, Result]])
    extends Parser[Name, Result]:
    def this(args: Parser[Name, Result]*) = this(args)

    override def apply(input: BufferedStream[Token[Name]]): Option[Result] =
        parsers.view.map(_.apply(input)).find(_.isDefined).map(_.get)

enum Range:
    case AtLeast(min: Int)
    case AtMost(max: Int)
    case Between(min: Int, max: Int)
    this match
        case Between(min, max) =>
            if min > max then
                throw IllegalArgumentException(
                  s"range out of order, min=$min, max=$max"
                )
        case _ => ()

    def contains(x: Int): Boolean =
        this match
            case AtLeast(min)      => x >= min
            case AtMost(max)       => x <= max
            case Between(min, max) => x >= min && x <= max

class Repeat[Name, Result](parser: Parser[Name, Result], range: Range)
    extends Parser[Name, List[Result]]:
    override def apply(
        input: BufferedStream[Token[Name]]
    ): Option[List[Result]] =
        @annotation.tailrec
        def f(current: List[Result]): Option[List[Result]] =
            val current_length_is_ok = range.contains(current.length)
            val next_length_is_ok = range.contains(current.length + 1)
            if current_length_is_ok && !next_length_is_ok then
                return Some(current)
            parser.apply(input) match
                case None => Some(current)
                case Some(value) =>
                    f(current :+ value)

        f(List()) match
            case None => None
            case Some(value) =>
                if range.contains(value.length) then Some(value)
                else None

extension [Name, Result](parser: Parser[Name, Result])
    def atLeast(min: Int): Parser[Name, List[Result]] =
        Repeat(parser, Range.AtLeast(min))

    def atMost(max: Int): Parser[Name, List[Result]] =
        Repeat(parser, Range.AtMost(max))

    def range(min: Int, max: Int): Parser[Name, List[Result]] =
        Repeat(parser, Range.Between(min, max))

    def optional(): Parser[Name, Option[Result]] =
        Repeat(parser, Range.Between(0, 1)).map(_ match
            case head :: next => Some(head)
            case Nil          => None
        )

private class MapParser[Name, IntermediateResult, Result](
    parser: Parser[Name, IntermediateResult],
    f: (result: IntermediateResult) => Result
) extends Parser[Name, Result]:
    override def apply(input: BufferedStream[Token[Name]]): Option[Result] =
        parser.apply(input).map(f)

extension [Name, IntermediateResult, Result](
    parser: Parser[Name, IntermediateResult]
)
    def map(f: (result: IntermediateResult) => Result): Parser[Name, Result] =
        MapParser(parser, f)

class Deferred[Name, Result] extends Parser[Name, Result]:
    private var parser: Option[Parser[Name, Result]] = None

    override def apply(input: BufferedStream[Token[Name]]): Option[Result] =
        parser match
            case None        => throw IllegalStateException("not resolved")
            case Some(value) => value.apply(input)

    def resolve(parser: Parser[Name, Result]) =
        if this.parser.isDefined then
            throw IllegalStateException("already resolved")
        this.parser = Some(parser)
