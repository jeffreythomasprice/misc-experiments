open Test
open Parsers

let m = string("foo")
list{
  ("success", "foobar", Some({result: "foo", remainder: "bar"})),
  ("failure", "fobar", None),
}->List.forEach(((name, input, expected)) =>
  test("string literals: " ++ name, () => {
    assertion((a, b) => {a == b}, m(input), expected)
  })
)

let m = list{string("foo"), string("bar"), string("baz")}->list
list{
  ("success", "foobarbaz_", Some({result: list{"foo", "bar", "baz"}, remainder: "_"})),
  ("failure on 1", "fobarbaz_", None),
  ("failure on 2", "foobabaz_", None),
  ("failure on 3", "foobarba_", None),
}->List.forEach(((name, input, expected)) =>
  test("list: " ++ name, () => {
    assertion((a, b) => {a == b}, m(input), expected)
  })
)

let m = tuple2(string("foo"), string("bar"))
list{
  ("success", "foobar_", Some({result: ("foo", "bar"), remainder: "_"})),
  ("failure on 1", "fobar_", None),
  ("failure on 2", "fooba_", None),
}->List.forEach(((name, input, expected)) =>
  test("tuple2: " ++ name, () => {
    assertion((a, b) => {a == b}, m(input), expected)
  })
)

let m = tuple3(string("foo"), string("bar"), string("baz"))
list{
  ("success", "foobarbaz_", Some({result: ("foo", "bar", "baz"), remainder: "_"})),
  ("failure on 1", "fobarbaz_", None),
  ("failure on 2", "foobabaz_", None),
  ("failure on 3", "foobarba_", None),
}->List.forEach(((name, input, expected)) =>
  test("tuple2: " ++ name, () => {
    assertion((a, b) => {a == b}, m(input), expected)
  })
)

let m = anyOf(list{string("foo"), string("bar"), string("baz")})
list{
  ("success on 1", "foo_", Some({result: "foo", remainder: "_"})),
  ("success on 2", "bar_", Some({result: "bar", remainder: "_"})),
  ("success on 3", "baz_", Some({result: "baz", remainder: "_"})),
  ("failure", "asdf_", None),
}->List.forEach(((name, input, expected)) =>
  test("anyOf: " ++ name, () => {
    assertion((a, b) => {a == b}, m(input), expected)
  })
)

let m = string("foo")->option
list{
  ("success", "foo_", Some({result: Some("foo"), remainder: "_"})),
  ("failure", "bar_", Some({result: None, remainder: "bar_"})),
}->List.forEach(((name, input, expected)) =>
  test("option: " ++ name, () => {
    assertion((a, b) => {a == b}, m(input), expected)
  })
)
