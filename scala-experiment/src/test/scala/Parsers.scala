class StringLiteralSuite extends munit.FunSuite {
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
        ),
        (
          "extension method, success",
          "Foo".toMatcher,
          "Foobar",
          Some(MatchResult(result = "Foo", remainder = "bar"))
        ),
        (
          "extension method, fail",
          "foo".toMatcher,
          "Foobar",
          None
        )
      )
    ) test(name) {
        assertEquals(m(input), expected)
    }
}

class CharRangeSuite extends munit.FunSuite {
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

class ListSuite extends munit.FunSuite {
    for (
      (name, m, input, expected) <- List(
        (
          "success",
          List(
            stringLiteral("aaa"),
            stringLiteral("bbb"),
            stringLiteral("ccc")
          ).toListMatcher,
          "aaabbbccc___",
          Some(
            MatchResult(result = List("aaa", "bbb", "ccc"), remainder = "___")
          )
        )
      )
    ) test(name) {
        assertEquals(m(input), expected)
    }
}

class Tuple2Suite extends munit.FunSuite {
    for (
      (name, m, input, expected) <- List(
        (
          "success",
          ("foo".toMatcher, "bar".toMatcher).toMatcher,
          "foobar_",
          Some(MatchResult(("foo", "bar"), "_"))
        ),
        (
          "fail on 1",
          ("foo".toMatcher, "bar".toMatcher).toMatcher,
          "fobar_",
          None
        ),
        (
          "fail on 2",
          ("foo".toMatcher, "bar".toMatcher).toMatcher,
          "fooba_",
          None
        )
      )
    ) test(name) {
        assertEquals(m(input), expected)
    }
}

class Tuple3Suite extends munit.FunSuite {
    for (
      (name, m, input, expected) <- List(
        (
          "success",
          ("aaa".toMatcher, "bbb".toMatcher, "ccc".toMatcher).toMatcher,
          "aaabbbccc___",
          Some(MatchResult(("aaa", "bbb", "ccc"), "___"))
        ),
        (
          "fail on 1",
          ("aaa".toMatcher, "bbb".toMatcher, "ccc".toMatcher).toMatcher,
          "aabbbccc___",
          None
        ),
        (
          "fail on 2",
          ("aaa".toMatcher, "bbb".toMatcher, "ccc".toMatcher).toMatcher,
          "aaabbccc___",
          None
        ),
        (
          "fail on 3",
          ("aaa".toMatcher, "bbb".toMatcher, "ccc".toMatcher).toMatcher,
          "aaabbbcc___",
          None
        )
      )
    ) test(name) {
        assertEquals(m(input), expected)
    }
}
