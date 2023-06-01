import java.io.BufferedReader
import java.io.StringReader
import scala.util.matching.Regex
class ParsersSuite extends munit.FunSuite:
    test("single token") {
        enum TokenType:
            case Foo
            case Bar
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .build()
        val parser = SingleToken(TokenType.Foo)
        assertEquals(
          parser.apply(
            BufferedStream(
              tokenizer.apply(BufferedReader(StringReader("foobar")))
            )
          ),
          Some(Token(TokenType.Foo, "foo", Position(row = 0, column = 0)))
        )
        assertEquals(
          parser.apply(
            BufferedStream(
              tokenizer.apply(BufferedReader(StringReader("barfoo")))
            )
          ),
          None
        )
    }

    test("sequence 2") {
        enum TokenType:
            case Foo
            case Bar
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .build()
        val parser =
            Sequence2(SingleToken(TokenType.Foo), SingleToken(TokenType.Bar))
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobar")))
                )
              ),
          Some(
            Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
            Token(TokenType.Bar, "bar", Position(row = 0, column = 3))
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("barbar")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoo")))
                )
              ),
          None
        )
    }

    test("sequence 3") {
        enum TokenType:
            case Foo
            case Bar
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .build()
        val parser = Sequence3(
          SingleToken(TokenType.Foo),
          SingleToken(TokenType.Bar),
          SingleToken(TokenType.Foo)
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobarfoo")))
                )
              ),
          Some(
            Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
            Token(TokenType.Bar, "bar", Position(row = 0, column = 3)),
            Token(TokenType.Foo, "foo", Position(row = 0, column = 6))
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("barbarfoo")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoofoo")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobarbar")))
                )
              ),
          None
        )
    }

    test("sequence 4") {
        enum TokenType:
            case Foo
            case Bar
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .build()
        val parser = Sequence4(
          SingleToken(TokenType.Foo),
          SingleToken(TokenType.Bar),
          SingleToken(TokenType.Foo),
          SingleToken(TokenType.Bar)
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobarfoobar")))
                )
              ),
          Some(
            Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
            Token(TokenType.Bar, "bar", Position(row = 0, column = 3)),
            Token(TokenType.Foo, "foo", Position(row = 0, column = 6)),
            Token(TokenType.Bar, "bar", Position(row = 0, column = 9))
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("barbarfoobar")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoofoobar")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobarbarbar")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobarfoofoo")))
                )
              ),
          None
        )
    }

    test("one of") {
        enum TokenType:
            case Foo
            case Bar
            case Baz
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .literal(TokenType.Baz, "baz")
            .build()
        var parser =
            OneOf(SingleToken(TokenType.Foo), SingleToken(TokenType.Bar))
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foo")))
                )
              ),
          Some(
            Token(TokenType.Foo, "foo", Position(row = 0, column = 0))
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("bar")))
                )
              ),
          Some(
            Token(TokenType.Bar, "bar", Position(row = 0, column = 0))
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("baz")))
                )
              ),
          None
        )
    }

    test("bad range") {
        assertEquals(
          intercept[IllegalArgumentException] {
              Range.Between(2, 1)
          }.getMessage(),
          "range out of order, min=2, max=1"
        )
    }

    test("repeat, at least") {
        enum TokenType:
            case Foo
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .build()
        var parser = SingleToken(TokenType.Foo).atLeast(2)
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoofoo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 3)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 6))
            )
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foo")))
                )
              ),
          None
        )
    }

    test("repeat, no more than") {
        enum TokenType:
            case Foo
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .build()
        var parser = SingleToken(TokenType.Foo).atMost(2)
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0))
            )
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 3))
            )
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoofoo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 3))
            )
          )
        )
    }

    test("repeat, between") {
        enum TokenType:
            case Foo
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .build()
        var parser = SingleToken(TokenType.Foo).range(2, 3)
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foo")))
                )
              ),
          None
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 3))
            )
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoofoo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 3)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 6))
            )
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foofoofoofoo")))
                )
              ),
          Some(
            List(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 3)),
              Token(TokenType.Foo, "foo", Position(row = 0, column = 6))
            )
          )
        )
    }

    test("optional") {
        enum TokenType:
            case Foo
            case Bar
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .build()
        var parser = SingleToken(TokenType.Foo).optional()
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foobar")))
                )
              ),
          Some(
            Some(
              Token(TokenType.Foo, "foo", Position(row = 0, column = 0))
            )
          )
        )
        assertEquals(
          parser
              .apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("bar")))
                )
              ),
          Some(
            None
          )
        )
    }

    test("map") {
        enum TokenType:
            case Number
            case Identifier
        val tokenizer = TokenizerBuilder()
            .regex(TokenType.Number, Regex("\\d+"))
            .literal(TokenType.Identifier, "foo")
            .build()
        val parser =
            SingleToken(TokenType.Number).map(result => result.value.toInt)
        assertEquals(
          parser.apply(
            BufferedStream(
              tokenizer.apply(BufferedReader(StringReader("123")))
            )
          ),
          Some(123)
        )
        assertEquals(
          parser.apply(
            BufferedStream(
              tokenizer.apply(BufferedReader(StringReader("foo")))
            )
          ),
          None
        )
    }

    test("deferred") {
        enum TokenType:
            case Foo
        val tokenizer = TokenizerBuilder()
            .literal(TokenType.Foo, "foo")
            .build()
        var parser = Deferred[TokenType, Token[TokenType]]()

        assertEquals(
          intercept[IllegalStateException] {
              parser.apply(
                BufferedStream(
                  tokenizer.apply(BufferedReader(StringReader("foo")))
                )
              )
          }.getMessage(),
          "not resolved"
        )

        parser.resolve(SingleToken(TokenType.Foo))

        assertEquals(
          parser.apply(
            BufferedStream(
              tokenizer.apply(BufferedReader(StringReader("foo")))
            )
          ),
          Some(
            Token(TokenType.Foo, "foo", Position(row = 0, column = 0))
          )
        )
    }
