import gleam/list
import gleam/option
import parser

pub fn string_literal_test() {
  let m = parser.string_literal("foo")
  [
    #(
      "foobar",
      option.Some(parser.MatchResult(result: "foo", remainder: "bar")),
    ),
    #("bar", option.None),
  ]
  |> list.each(fn(x) {
    let #(input, expected) = x
    let actual = m(input)
    assert actual == expected
  })
}
