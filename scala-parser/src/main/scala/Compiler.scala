import java.io.BufferedReader
import scala.collection.mutable.ListBuffer
import java.io.OptionalDataException
class Compiler[Name, Result](
    tokenizer: Tokenizer[Name],
    tokenBufferLength: Option[Int],
    intermediatePhases: Iterable[Parser[Name, Stream[Token[Name]]]],
    finalPhase: Parser[Name, Result]
)
def apply(input: BufferedReader) = ???

class CompilerBuilder[Name, Result]:
    private var _tokenizer: Option[Tokenizer[Name]] = None
    private var _tokenBufferLength: Option[Int] = None
    private val _intermediatePhases =
        ListBuffer[Parser[Name, Stream[Token[Name]]]]()
    private var _finalPhase: Option[Parser[Name, Result]] = None

    def tokenizer(tokenizer: Tokenizer[Name]) =
        _tokenizer = Some(tokenizer)
        this

    def tokenBufferLength(
        tokenBufferLength: Option[Int]
    ): CompilerBuilder[Name, Result] =
        _tokenBufferLength = tokenBufferLength
        this

    def addIntermediatePhase(
        parser: Parser[Name, Stream[Token[Name]]]
    ): CompilerBuilder[Name, Result] =
        _intermediatePhases += parser
        this

    def finalPhase(
        parser: Parser[Name, Result]
    ): CompilerBuilder[Name, Result] =
        _finalPhase = Some(parser)
        this

    def build(): Compiler[Name, Result] =
        Compiler[Name, Result](
          _tokenizer.get,
          _tokenBufferLength,
          _intermediatePhases,
          _finalPhase.get
        )
