import java.io.BufferedReader
import java.io.StringReader
import scala.collection.mutable.ListBuffer
import scala.util.matching.Regex

case class Position(val row: Int, val column: Int):
    def +(s: String) =
        val lines = s.split("\n")
        if lines.length == 1 then
            Position(row = row, column = column + s.length)
        else Position(row = row + lines.length - 1, column = lines.last.length)

case class Token[Name](
    val name: Name,
    val value: String,
    val position: Position
)

sealed trait TokenMatcher[Name]:
    def apply(
        input: BufferedReader,
        currentPosition: Position
    ): Option[Token[Name]]

case class StringTokenMatcher[Name](
    val name: Name,
    val value: String,
    val caseInsensitive: Boolean
) extends TokenMatcher[Name]:
    override def apply(
        input: BufferedReader,
        currentPosition: Position
    ): Option[Token[Name]] =
        val result = input.peek(value.length()) match
            case None => None
            case Some(peek) =>
                if caseInsensitive then
                    if value.equalsIgnoreCase(peek) then Some((name, peek))
                    else None
                else if value == peek then Some((name, peek))
                else None
        result match
            case None => None
            case Some((name, peek)) =>
                input.skip(peek.length())
                Some(Token(name, peek, currentPosition))

case class RegexTokenMatcher[Name](
    val name: Name,
    val regex: Regex,
    val peekLength: Int
) extends TokenMatcher[Name]:
    override def apply(
        input: BufferedReader,
        currentPosition: Position
    ): Option[Token[Name]] =
        input
            .peek(peekLength)
            .flatMap(regex.findFirstMatchIn(_))
            .filter(_.start == 0)
            .map(m =>
                input.skip(m.matched.length)
                Token(name, m.matched, currentPosition)
            )

class Tokenizer[Name](private val matchers: Iterable[TokenMatcher[Name]]):
    def apply(input: BufferedReader): Stream[Token[Name]] =
        var position = Position(0, 0)
        CallbackStream(() =>
            matchers.view.map(_.apply(input, position)).find(_.isDefined) match
                case None => None
                case Some(value) =>
                    value.foreach(position += _.value)
                    value
        )

class TokenizerBuilder[Name]:
    private val matchers = ListBuffer[TokenMatcher[Name]]()

    private var _peekLength = 1024

    def build() = Tokenizer(matchers.toList)

    def append(matcher: TokenMatcher[Name]): TokenizerBuilder[Name] =
        matchers += matcher
        this

    def literal(
        name: Name,
        value: String,
        caseInsensitive: Boolean = false
    ): TokenizerBuilder[Name] =
        append(StringTokenMatcher(name, value, caseInsensitive))

    def regex(
        name: Name,
        regex: Regex,
        peekLength: Int = this.peekLength()
    ): TokenizerBuilder[Name] =
        append(RegexTokenMatcher(name, regex, peekLength))

    def peekLength(): Int = _peekLength

    def peekLength(peekLength: Int): TokenizerBuilder[Name] =
        _peekLength = peekLength
        this
