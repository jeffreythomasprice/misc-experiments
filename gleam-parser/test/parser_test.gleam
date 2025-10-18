import gleam/list
import gleam/option
import gleam/string
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

pub fn utf_codepoint_range_test() {
  let assert Ok(min) = string.to_utf_codepoints("a") |> list.first
  let assert Ok(max) = string.to_utf_codepoints("z") |> list.first
  let assert Ok(j) = string.to_utf_codepoints("j") |> list.first
  let m = parser.utf_codepoint_range(min, max)
  [
    #(
      "success",
      "j_",
      option.Some(parser.MatchResult(result: j, remainder: "_")),
    ),
    #("failure", "!_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn char_range_test() {
  let m = parser.char_range("a", "z")
  [
    #(
      "success",
      "j_",
      option.Some(parser.MatchResult(result: "j", remainder: "_")),
    ),
    #("failure", "!_", option.None),
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

pub fn any_of_test() {
  let m =
    parser.any_of([
      parser.string("foo"),
      parser.string("bar"),
      parser.string("baz"),
    ])
  [
    #(
      "success on 1",
      "foo_",
      option.Some(parser.MatchResult(result: "foo", remainder: "_")),
    ),
    #(
      "success on 2",
      "bar_",
      option.Some(parser.MatchResult(result: "bar", remainder: "_")),
    ),
    #(
      "success on 3",
      "baz_",
      option.Some(parser.MatchResult(result: "baz", remainder: "_")),
    ),
    #("failure", "asdf_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn option_test() {
  let m = parser.option(parser.string("foo"))
  [
    #(
      "success",
      "foo_",
      option.Some(parser.MatchResult(result: option.Some("foo"), remainder: "_")),
    ),
    #(
      "failure",
      "bar_",
      option.Some(parser.MatchResult(result: option.None, remainder: "bar_")),
    ),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn at_least_test() {
  [
    #(
      "at least 0, input had 0",
      parser.at_least(parser.string("foo"), 0),
      "_",
      option.Some(parser.MatchResult(result: [], remainder: "_")),
    ),
    #(
      "at least 0, input had 1",
      parser.at_least(parser.string("foo"), 0),
      "foo_",
      option.Some(parser.MatchResult(result: ["foo"], remainder: "_")),
    ),
    #(
      "at least 0, input had 2",
      parser.at_least(parser.string("foo"), 0),
      "foofoo_",
      option.Some(parser.MatchResult(result: ["foo", "foo"], remainder: "_")),
    ),
    #(
      "at least 1, input had 0",
      parser.at_least(parser.string("foo"), 1),
      "_",
      option.None,
    ),
    #(
      "at least 1, input had 1",
      parser.at_least(parser.string("foo"), 1),
      "foo_",
      option.Some(parser.MatchResult(result: ["foo"], remainder: "_")),
    ),
    #(
      "at least 1, input had 2",
      parser.at_least(parser.string("foo"), 1),
      "foofoo_",
      option.Some(parser.MatchResult(result: ["foo", "foo"], remainder: "_")),
    ),
    #(
      "at least 2, input had 0",
      parser.at_least(
        parser.any_of([parser.string("foo"), parser.string("bar")]),
        2,
      ),
      "_",
      option.None,
    ),
    #(
      "at least 2, input had 1",
      parser.at_least(
        parser.any_of([parser.string("foo"), parser.string("bar")]),
        2,
      ),
      "foo_",
      option.None,
    ),
    #(
      "at least 2, input had 2",
      parser.at_least(
        parser.any_of([parser.string("foo"), parser.string("bar")]),
        2,
      ),
      "foobar_",
      option.Some(parser.MatchResult(result: ["foo", "bar"], remainder: "_")),
    ),
    #(
      "at least 2, input had 3",
      parser.at_least(
        parser.any_of([parser.string("foo"), parser.string("bar")]),
        2,
      ),
      "foobarfoo_",
      option.Some(parser.MatchResult(
        result: ["foo", "bar", "foo"],
        remainder: "_",
      )),
    ),
  ]
  |> list.each(fn(x) {
    let #(name, m, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn skip_prefix_test() {
  let m = parser.skip_prefix(parser.string("("), parser.string("foo"))
  [
    #(
      "success",
      "(foo_",
      option.Some(parser.MatchResult(result: "foo", remainder: "_")),
    ),
    #("failure", "foo_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn skip_suffix_test() {
  let m = parser.skip_suffix(parser.string("foo"), parser.string(")"))
  [
    #(
      "success",
      "foo)_",
      option.Some(parser.MatchResult(result: "foo", remainder: "_")),
    ),
    #("failure", "foo_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn skip_prefix_and_suffix_test() {
  let m =
    parser.skip_prefix_and_suffix(
      parser.string("("),
      parser.string("foo"),
      parser.string(")"),
    )
  [
    #(
      "success",
      "(foo)_",
      option.Some(parser.MatchResult(result: "foo", remainder: "_")),
    ),
    #("failure, missing prefix", "foo)_", option.None),
    #("failure, missing suffix", "(foo_", option.None),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}

pub fn map_test() {
  let m = parser.at_least(parser.string("foo"), 1) |> parser.map(list.length)
  [
    #("fail with 0 inputs", "_", option.None),
    #(
      "success with 1 inputs",
      "foo_",
      option.Some(parser.MatchResult(result: 1, remainder: "_")),
    ),
    #(
      "success with 2 inputs",
      "foofoo_",
      option.Some(parser.MatchResult(result: 2, remainder: "_")),
    ),
  ]
  |> list.each(fn(x) {
    let #(name, input, expected) = x
    let actual = m(input)
    assert actual == expected as name
  })
}
