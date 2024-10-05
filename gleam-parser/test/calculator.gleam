import gleam/float
import gleam/option
import gleam/string
import parser

pub fn skip_whitespace(p: parser.Parser(t)) -> parser.Parser(t) {
  p
  |> parser.skip_prefix(skip: parser.at_least(
    parser.any4(
      parser.string(" "),
      parser.string("\t"),
      parser.string("\n"),
      parser.string("\r"),
    ),
    0,
  ))
}

pub fn number() -> parser.Parser(Float) {
  // https://stackoverflow.com/a/13340826
  // -?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?

  // breaking that up into parts we get:
  // integer part: -?(?:0|[1-9]\d*)
  // optional fractional part: (?:\.\d+)?
  // optional exponent part: (?:[eE][+-]?\d+)?

  let assert Ok(digit_1_to_9) = parser.char_range("1", "9")
  let assert Ok(digit_0_to_9) = parser.char_range("0", "9")

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
}
