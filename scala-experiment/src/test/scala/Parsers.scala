class StringLiterals extends munit.FunSuite {
    for (
      (name, m, input, expected) <- List(
        (
          "default case-sensitivity, success",
          stringLiteral("Foo"),
          "Foobar",
          Some(MatchResult(result = "Foo", remainder = "bar"))
        ),
        (
          "default case-sensitivity, fail",
          stringLiteral("foo"),
          "Foobar",
          None
        ),
        (
          "case-sensitive, success",
          stringLiteral("Foo", caseSensitive = true),
          "Foobar",
          Some(MatchResult(result = "Foo", remainder = "bar"))
        ),
        (
          "case-sensitive, fail",
          stringLiteral("foo", caseSensitive = true),
          "Foobar",
          None
        ),
        (
          "case-insensitive, success",
          stringLiteral("Foo", caseSensitive = false),
          "Foobar",
          Some(MatchResult(result = "Foo", remainder = "bar"))
        ),
        (
          "case-insensitive, fail",
          stringLiteral("foo", caseSensitive = false),
          "Foobar",
          Some(MatchResult(result = "Foo", remainder = "bar"))
        )
      )
    ) test(name) {
        assertEquals(m(input), expected)
    }
}

class CharRange extends munit.FunSuite {
    for (
      (lower, upper, input, expected) <- List(
        (
          'a',
          'z',
          "foo",
          Some(MatchResult(result = 'f', remainder = "oo"))
        ),
        (
          'a',
          'z',
          "123",
          None
        )
      )
    ) test(s"[$lower..$upper] => $expected") {
        assertEquals(charRange(lower, upper)(input), expected)
    }
}
