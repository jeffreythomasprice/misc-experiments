import gleam/option
import gleam/string
import gleeunit
import gleeunit/should
import parser

pub fn main() {
  gleeunit.main()
}

pub fn string_test() {
  let p = parser.string("foo")
  p("foobar") |> should.equal(Ok(#("bar", "foo")))
  p("foo") |> should.equal(Ok(#("", "foo")))
  p("far") |> should.equal(Error(Nil))
}

pub fn map_test() {
  let p = parser.string("foo") |> parser.map(fn(_) { 42 })
  p("foobar") |> should.equal(Ok(#("bar", 42)))
  p("foo") |> should.equal(Ok(#("", 42)))
  p("far") |> should.equal(Error(Nil))
}

pub fn flatten_test() {
  let p =
    parser.any2(
      parser.string("foo")
        |> parser.map(fn(_) { Ok(1) })
        |> parser.flatten,
      parser.string("bar")
        |> parser.map(fn(_) { Error(Nil) })
        |> parser.flatten,
    )
  p("foo__") |> should.equal(Ok(#("__", 1)))
  p("bar__") |> should.equal(Error(Nil))
  p("baz__") |> should.equal(Error(Nil))
}

pub fn utf_codepoint_range_test() {
  let assert [c1] = string.to_utf_codepoints("0")
  let assert [c2] = string.to_utf_codepoints("9")
  let p = parser.utf_codepoint_range(c1, c2)
  p("0abc") |> should.equal(Ok(#("abc", "0")))
  p("1abc") |> should.equal(Ok(#("abc", "1")))
  p("2abc") |> should.equal(Ok(#("abc", "2")))
  p("3abc") |> should.equal(Ok(#("abc", "3")))
  p("4abc") |> should.equal(Ok(#("abc", "4")))
  p("5abc") |> should.equal(Ok(#("abc", "5")))
  p("6abc") |> should.equal(Ok(#("abc", "6")))
  p("7abc") |> should.equal(Ok(#("abc", "7")))
  p("8abc") |> should.equal(Ok(#("abc", "8")))
  p("9abc") |> should.equal(Ok(#("abc", "9")))
  p("abc") |> should.equal(Error(Nil))
  p("") |> should.equal(Error(Nil))
}

pub fn char_range_test() {
  let assert Ok(p) = parser.char_range("0", "9")
  p("0abc") |> should.equal(Ok(#("abc", "0")))
  p("1abc") |> should.equal(Ok(#("abc", "1")))
  p("2abc") |> should.equal(Ok(#("abc", "2")))
  p("3abc") |> should.equal(Ok(#("abc", "3")))
  p("4abc") |> should.equal(Ok(#("abc", "4")))
  p("5abc") |> should.equal(Ok(#("abc", "5")))
  p("6abc") |> should.equal(Ok(#("abc", "6")))
  p("7abc") |> should.equal(Ok(#("abc", "7")))
  p("8abc") |> should.equal(Ok(#("abc", "8")))
  p("9abc") |> should.equal(Ok(#("abc", "9")))
  p("abc") |> should.equal(Error(Nil))
  p("") |> should.equal(Error(Nil))

  parser.char_range("", "9") |> should.equal(Error(Nil))
  parser.char_range("0", "") |> should.equal(Error(Nil))
  parser.char_range("00", "9") |> should.equal(Error(Nil))
  parser.char_range("0", "99") |> should.equal(Error(Nil))

  let assert Ok(p) = parser.char_range("9", "0")
  p("0abc") |> should.equal(Error(Nil))
  p("1abc") |> should.equal(Error(Nil))
  p("2abc") |> should.equal(Error(Nil))
  p("3abc") |> should.equal(Error(Nil))
  p("4abc") |> should.equal(Error(Nil))
  p("5abc") |> should.equal(Error(Nil))
  p("6abc") |> should.equal(Error(Nil))
  p("7abc") |> should.equal(Error(Nil))
  p("8abc") |> should.equal(Error(Nil))
  p("9abc") |> should.equal(Error(Nil))
  p("abc") |> should.equal(Error(Nil))
  p("") |> should.equal(Error(Nil))
}

// TODO regex test

pub fn seq2_test() {
  let p = parser.seq2(parser.string("a"), parser.string("b"))
  p("ab__") |> should.equal(Ok(#("__", #("a", "b"))))
  p("ac__") |> should.equal(Error(Nil))
}

pub fn seq3_test() {
  let p =
    parser.seq3(parser.string("a"), parser.string("b"), parser.string("c"))
  p("abc__") |> should.equal(Ok(#("__", #("a", "b", "c"))))
  p("abd__") |> should.equal(Error(Nil))
}

pub fn seq4_test() {
  let p =
    parser.seq4(
      parser.string("a"),
      parser.string("b"),
      parser.string("c"),
      parser.string("d"),
    )
  p("abcd__") |> should.equal(Ok(#("__", #("a", "b", "c", "d"))))
  p("abce__") |> should.equal(Error(Nil))
}

pub fn any2_test() {
  let p = parser.any2(parser.string("a"), parser.string("b"))
  p("a__") |> should.equal(Ok(#("__", "a")))
  p("b__") |> should.equal(Ok(#("__", "b")))
  p("c__") |> should.equal(Error(Nil))
}

pub fn any3_test() {
  let p =
    parser.any3(parser.string("a"), parser.string("b"), parser.string("c"))
  p("a__") |> should.equal(Ok(#("__", "a")))
  p("b__") |> should.equal(Ok(#("__", "b")))
  p("c__") |> should.equal(Ok(#("__", "c")))
  p("d__") |> should.equal(Error(Nil))
}

pub fn any4_test() {
  let p =
    parser.any4(
      parser.string("a"),
      parser.string("b"),
      parser.string("c"),
      parser.string("d"),
    )
  p("a__") |> should.equal(Ok(#("__", "a")))
  p("b__") |> should.equal(Ok(#("__", "b")))
  p("c__") |> should.equal(Ok(#("__", "c")))
  p("d__") |> should.equal(Ok(#("__", "d")))
  p("e__") |> should.equal(Error(Nil))
}

pub fn repeat_test() {
  let p =
    parser.repeat(parser.string("foo"), parser.Bounded(2), parser.Bounded(3))
  p("") |> should.equal(Error(Nil))
  p("foo") |> should.equal(Error(Nil))
  p("foofoo") |> should.equal(Ok(#("", ["foo", "foo"])))
  p("foofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo"])))
  p("foofoofoofoo") |> should.equal(Ok(#("foo", ["foo", "foo", "foo"])))

  let p =
    parser.repeat(parser.string("foo"), parser.Unbounded, parser.Bounded(3))
  p("") |> should.equal(Ok(#("", [])))
  p("foo") |> should.equal(Ok(#("", ["foo"])))
  p("foofoo") |> should.equal(Ok(#("", ["foo", "foo"])))
  p("foofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo"])))
  p("foofoofoofoo") |> should.equal(Ok(#("foo", ["foo", "foo", "foo"])))

  let p =
    parser.repeat(parser.string("foo"), parser.Bounded(2), parser.Unbounded)
  p("") |> should.equal(Error(Nil))
  p("foo") |> should.equal(Error(Nil))
  p("foofoo") |> should.equal(Ok(#("", ["foo", "foo"])))
  p("foofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo"])))
  p("foofoofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo", "foo"])))

  let p =
    parser.repeat(parser.string("foo"), parser.Unbounded, parser.Unbounded)
  p("") |> should.equal(Ok(#("", [])))
  p("foo") |> should.equal(Ok(#("", ["foo"])))
  p("foofoo") |> should.equal(Ok(#("", ["foo", "foo"])))
  p("foofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo"])))
  p("foofoofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo", "foo"])))

  // prove they come out in the right order

  let p =
    parser.repeat(
      parser.any3(
        parser.string("foo"),
        parser.string("bar"),
        parser.string("baz"),
      ),
      parser.Unbounded,
      parser.Unbounded,
    )
  p("foobarbaz") |> should.equal(Ok(#("", ["foo", "bar", "baz"])))
}

pub fn at_least_test() {
  let p = parser.at_least(parser.string("foo"), 1)
  p("") |> should.equal(Error(Nil))
  p("foo") |> should.equal(Ok(#("", ["foo"])))
  p("foofoo") |> should.equal(Ok(#("", ["foo", "foo"])))
  p("foofoofoo") |> should.equal(Ok(#("", ["foo", "foo", "foo"])))
}

pub fn at_most_test() {
  let p = parser.at_most(parser.string("foo"), 2)
  p("") |> should.equal(Ok(#("", [])))
  p("foo") |> should.equal(Ok(#("", ["foo"])))
  p("foofoo") |> should.equal(Ok(#("", ["foo", "foo"])))
  p("foofoofoo") |> should.equal(Ok(#("foo", ["foo", "foo"])))
}

pub fn optional_test() {
  let p = parser.optional(parser.string("foo"))
  p("foobar") |> should.equal(Ok(#("bar", option.Some("foo"))))
  p("bar") |> should.equal(Ok(#("bar", option.None)))
}

pub fn skip_prefix_test() {
  let p = parser.string("foo") |> parser.skip_prefix(parser.string("bar"), _)
  p("barfoo__") |> should.equal(Ok(#("__", "foo")))
  p("foo__") |> should.equal(Error(Nil))
}
// TODO skip_suffix_test
// TODO skip_prefix_and_suffix_test
