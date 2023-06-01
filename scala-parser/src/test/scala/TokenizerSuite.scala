import java.io.BufferedReader
import java.io.StringReader
import scala.util.matching.Regex
class TokenizerSuite extends munit.FunSuite:
    test("position") {
        assertEquals(
          Position(row = 0, column = 0) + "foo",
          Position(row = 0, column = 3)
        )
        assertEquals(
          Position(row = 0, column = 0) + "foobar",
          Position(row = 0, column = 6)
        )
        assertEquals(
          Position(row = 2, column = 5) + "foo",
          Position(row = 2, column = 8)
        )
        assertEquals(
          Position(row = 0, column = 0) + "foo\nbar\nbaz",
          Position(row = 2, column = 3)
        )
        assertEquals(
          Position(row = 2, column = 5) + "foo\nbar\nbaz",
          Position(row = 4, column = 3)
        )
        assertEquals(
          Position(row = 0, column = 0) + "",
          Position(row = 0, column = 0)
        )
        assertEquals(
          Position(row = 2, column = 5) + "",
          Position(row = 2, column = 5)
        )
    }

    test("string literals") {
        enum TokenType:
            case Foo
            case Bar
            case Baz
        val tokenizer = TokenizerBuilder[TokenType]
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "BAR", caseInsensitive = true)
            .literal(TokenType.Baz, "BAZ", caseInsensitive = false)
            .build()
        assertEquals(
          tokenizer
              .apply(BufferedReader(StringReader("BAZbarfoo")))
              .iterator
              .toList,
          List(
            Token(TokenType.Baz, "BAZ", Position(row = 0, column = 0)),
            Token(TokenType.Bar, "bar", Position(row = 0, column = 3)),
            Token(TokenType.Foo, "foo", Position(row = 0, column = 6))
          )
        )
        assertEquals(
          tokenizer
              .apply(BufferedReader(StringReader("barBaRBAR")))
              .iterator
              .toList,
          List(
            Token(TokenType.Bar, "bar", Position(row = 0, column = 0)),
            Token(TokenType.Bar, "BaR", Position(row = 0, column = 3)),
            Token(TokenType.Bar, "BAR", Position(row = 0, column = 6))
          )
        )
        assertEquals(
          tokenizer
              .apply(BufferedReader(StringReader("FOO")))
              .iterator
              .toList,
          List()
        )
        assertEquals(
          tokenizer
              .apply(BufferedReader(StringReader("baz")))
              .iterator
              .toList,
          List()
        )
    }

    test("regex") {
        enum TokenType:
            case Digits
            case Letters
        val tokenizer =
            TokenizerBuilder[TokenType]
                .regex(TokenType.Digits, Regex("[0-9]+"))
                .regex(TokenType.Letters, Regex("[a-z]+"))
                .build()
        assertEquals(
          tokenizer
              .apply(BufferedReader(StringReader("123abc456def")))
              .iterator
              .toList,
          List(
            Token(TokenType.Digits, "123", Position(row = 0, column = 0)),
            Token(TokenType.Letters, "abc", Position(row = 0, column = 3)),
            Token(TokenType.Digits, "456", Position(row = 0, column = 6)),
            Token(TokenType.Letters, "def", Position(row = 0, column = 9))
          )
        )
        assertEquals(
          tokenizer
              .apply(BufferedReader(StringReader("abcd11")))
              .iterator
              .toList,
          List(
            Token(TokenType.Letters, "abcd", Position(row = 0, column = 0)),
            Token(TokenType.Digits, "11", Position(row = 0, column = 4))
          )
        )
    }

    test("leftover reader content") {
        enum TokenType:
            case Foo
            case Bar
        val tokenizer = TokenizerBuilder[TokenType]
            .literal(TokenType.Foo, "foo")
            .literal(TokenType.Bar, "bar")
            .build()
        val reader = BufferedReader(StringReader("foobar123"))
        assertEquals(
          tokenizer
              .apply(reader)
              .iterator
              .toList,
          List(
            Token(TokenType.Foo, "foo", Position(row = 0, column = 0)),
            Token(TokenType.Bar, "bar", Position(row = 0, column = 3))
          )
        )
        val buffer = new Array[Char](100)
        assertEquals(reader.read(buffer), 3)
        assertEquals(
          buffer.slice(0, 3).toList,
          "123".toCharArray().toList
        )
    }
