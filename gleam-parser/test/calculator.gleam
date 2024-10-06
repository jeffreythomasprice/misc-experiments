import gleam/float
import gleam/list
import gleam/option
import gleam/string
import parser

pub type Node {
  Number(Float)
  Add(Node, Node)
  Subtract(Node, Node)
  Multiply(Node, Node)
  Divide(Node, Node)
  Negate(Node)
}

pub fn eval(n: Node) -> Float {
  case n {
    Number(result) -> result
    Add(left, right) -> eval(left) +. eval(right)
    Subtract(left, right) -> eval(left) -. eval(right)
    Multiply(left, right) -> eval(left) *. eval(right)
    Divide(left, right) -> eval(left) /. eval(right)
    Negate(value) -> 0.0 -. eval(value)
  }
}

pub fn skip_whitespace(p: parser.Parser(t)) -> parser.Parser(t) {
  p
  |> parser.skip_prefix(
    parser.at_least(
      parser.any4(
        parser.string(" "),
        parser.string("\t"),
        parser.string("\n"),
        parser.string("\r"),
      ),
      0,
    ),
    _,
  )
}

// TODO it seems inefficient to recreate every parser every time for stuff below here

pub fn number(input: String) -> Result(#(String, Float), Nil) {
  // https://stackoverflow.com/a/13340826
  // -?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?

  // breaking that up into parts we get:
  // integer part: -?(?:0|[1-9]\d*)
  // optional fractional part: (?:\.\d+)?
  // optional exponent part: (?:[eE][+-]?\d+)?

  let assert Ok(digit_1_to_9) = parser.char_range("1", "9")
  let assert Ok(digit_0_to_9) = parser.char_range("0", "9")

  let parser =
    parser.seq3(
      // integer part: -?(?:0|[1-9]\d*)
      parser.seq2(
        // leading negative sign
        parser.optional(parser.string("-")),
        parser.any2(
          // a single zero
          parser.string("0"),
          // a number that starts with a non-zero and then any number of other digits
          parser.seq2(digit_1_to_9, parser.at_least(digit_0_to_9, 0))
            // and turn it back into a single string
            |> parser.map(fn(result) {
              let #(first, remainder) = result
              string.concat([first, ..remainder])
            }),
        ),
      )
        // and turn it back into a single string
        |> parser.map(fn(result) {
          let #(negative_sign, remainder) = result
          option.unwrap(negative_sign, "") <> remainder
        }),
      // fractional part: (?:\.\d+)?
      parser.optional(parser.seq2(
        parser.string("."),
        parser.at_least(digit_0_to_9, 1),
      ))
        // and turn it back into a single string
        |> parser.map(fn(result) {
          case result {
            // gleam floats treat the fractional part as mandatory, so put a placeholder in therer
            option.None -> ".0"
            option.Some(#(dot, remainder)) -> dot <> string.concat(remainder)
          }
        }),
      // exponent part: (?:[eE][+-]?\d+)?
      parser.optional(parser.seq3(
        parser.any2(parser.string("e"), parser.string("E")),
        parser.optional(parser.any2(parser.string("+"), parser.string("-"))),
        parser.at_least(digit_0_to_9, 1),
      ))
        // and turn it back into a single string
        |> parser.map(fn(result) {
          case result {
            option.None -> ""
            option.Some(#(a, b, c)) -> {
              a <> option.unwrap(b, "") <> string.concat(c)
            }
          }
        }),
    )
    // and turn it back into a single string
    |> parser.map(fn(result) {
      let #(a, b, c) = result
      a <> b <> c
    })
    // and turn it into a float
    |> parser.map(fn(result) { float.parse(result) })
    |> parser.flatten

  parser(input)
}

fn number_node(input: String) -> Result(#(String, Node), Nil) {
  let parser =
    number |> skip_whitespace |> parser.map(fn(result) { Number(result) })
  parser(input)
}

fn negate(input: String) -> Result(#(String, Node), Nil) {
  let parser =
    number_node
    |> parser.skip_prefix(parser.string("-") |> skip_whitespace, _)
    |> parser.map(fn(result) { Negate(result) })
  parser(input)
}

fn term(input: String) -> Result(#(String, Node), Nil) {
  let parser =
    parser.any3(
      parser.skip_prefix_and_suffix(
        parser.string("(") |> skip_whitespace,
        expression,
        parser.string(")") |> skip_whitespace,
      ),
      negate,
      number_node,
    )
  parser(input)
}

fn binary_op(
  p1: parser.Parser(t),
  p2: parser.Parser(fn(t, t) -> t),
) -> parser.Parser(t) {
  parser.seq2(p1, parser.at_least(parser.seq2(p2, p1), 0))
  |> parser.map(fn(result) {
    let #(first, remainder) = result
    list.fold(remainder, first, fn(left, op_and_right) {
      let #(op, right) = op_and_right
      op(left, right)
    })
  })
}

fn multiply_or_divide(input: String) -> Result(#(String, Node), Nil) {
  let parser =
    binary_op(
      term,
      parser.any2(
        parser.string("*") |> skip_whitespace |> parser.map(fn(_) { Multiply }),
        parser.string("/") |> skip_whitespace |> parser.map(fn(_) { Divide }),
      ),
    )
  parser(input)
}

fn add_or_subtract(input: String) -> Result(#(String, Node), Nil) {
  let parser =
    binary_op(
      multiply_or_divide,
      parser.any2(
        parser.string("+") |> skip_whitespace |> parser.map(fn(_) { Add }),
        parser.string("-") |> skip_whitespace |> parser.map(fn(_) { Subtract }),
      ),
    )
  parser(input)
}

pub fn expression(input: String) -> Result(#(String, Node), Nil) {
  add_or_subtract(input)
}
