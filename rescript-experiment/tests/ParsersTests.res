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
