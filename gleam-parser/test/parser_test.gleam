import gleam/list
import gleam/option
import parser

pub fn string_test() {
  let m = parser.string("foo")
  [
    #(
      "success",
      "foobar",
      option.Some(parser.MatchResult(result: "foo", remainder: "bar")),
    ),
    #("failure", "bar", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn list_test() {
  let m =
    parser.list([
      parser.string("foo"),
      parser.string("bar"),
      parser.string("baz"),
    ])
  [
    #(
      "success",
      "foobarbaz_",
      option.Some(parser.MatchResult(
        result: ["foo", "bar", "baz"],
        remainder: "_",
      )),
    ),
    #("failure on 1", "fobarbaz_", option.None),
    #("failure on 2", "foobabaz_", option.None),
    #("failure on 3", "foobarba_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn tuple2_test() {
  let m = parser.tuple2(parser.string("foo"), parser.string("bar"))
  [
    #(
      "success",
      "foobar_",
      option.Some(parser.MatchResult(result: #("foo", "bar"), remainder: "_")),
    ),
    #("failure on 1", "fobar_", option.None),
    #("failure on 2", "fooba_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn tuple3_test() {
  let m =
    parser.tuple3(
      parser.string("foo"),
      parser.string("bar"),
      parser.string("baz"),
    )
  [
    #(
      "success",
      "foobarbaz_",
      option.Some(parser.MatchResult(
        result: #("foo", "bar", "baz"),
        remainder: "_",
      )),
    ),
    #("failure on 1", "fobarbaz_", option.None),
    #("failure on 2", "foobabaz_", option.None),
    #("failure on 3", "foobarba_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}
