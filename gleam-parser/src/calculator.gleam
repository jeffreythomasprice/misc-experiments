import gleam/float
import gleam/int
import gleam/list
import gleam/option
import gleam/string
import parser

pub type ASTNode {
  Number(Float)
  Negate(ASTNode)
  Add(left: ASTNode, right: ASTNode)
  Subtract(left: ASTNode, right: ASTNode)
  Multiply(left: ASTNode, right: ASTNode)
  Divide(left: ASTNode, right: ASTNode)
}

pub fn evaluate(input: String) -> option.Option(Float) {
  case expression(input) {
    option.None -> option.None
    option.Some(parser.MatchResult(result:, remainder: _)) ->
      option.Some(value_of_ast_node(result))
  }
}

fn value_of_ast_node(node: ASTNode) -> Float {
  case node {
    Number(value) -> value
    Negate(value) -> 0.0 -. value_of_ast_node(value)
    Add(left:, right:) -> value_of_ast_node(left) +. value_of_ast_node(right)
    Subtract(left:, right:) ->
      value_of_ast_node(left) -. value_of_ast_node(right)
    Multiply(left:, right:) ->
      value_of_ast_node(left) *. value_of_ast_node(right)
    Divide(left:, right:) -> value_of_ast_node(left) /. value_of_ast_node(right)
  }
}

fn expression(input: String) -> option.Option(parser.MatchResult(ASTNode)) {
  add_or_subtract_operation(input)
}

fn add_or_subtract_operation(
  input: String,
) -> option.Option(parser.MatchResult(ASTNode)) {
  // TODO de-duplicate with multiply_or_divide node, make a generic binary_op
  // multiply_or_divide_operation (("+" | "-") multiply_or_divide_operation)*
  case
    parser.tuple2(
      multiply_or_divide_operation,
      parser.tuple2(
        parser.any_of([
          string("+") |> parser.map(fn(_) { Add }),
          string("-") |> parser.map(fn(_) { Subtract }),
        ]),
        multiply_or_divide_operation,
      )
        |> parser.at_least(0),
    )(input)
  {
    option.None -> option.None
    option.Some(parser.MatchResult(result: #(first, rest), remainder:)) ->
      option.Some(parser.MatchResult(
        result: list.fold(rest, first, fn(left, next) {
          let #(op, right) = next
          op(left, right)
        }),
        remainder:,
      ))
  }
}

fn multiply_or_divide_operation(
  input: String,
) -> option.Option(parser.MatchResult(ASTNode)) {
  // term (("*" | "/") term)*
  case
    parser.tuple2(
      term,
      parser.tuple2(
        parser.any_of([
          string("*") |> parser.map(fn(_) { Multiply }),
          string("/") |> parser.map(fn(_) { Divide }),
        ]),
        term,
      )
        |> parser.at_least(0),
    )(input)
  {
    option.None -> option.None
    option.Some(parser.MatchResult(result: #(first, rest), remainder:)) ->
      option.Some(parser.MatchResult(
        result: list.fold(rest, first, fn(left, next) {
          let #(op, right) = next
          op(left, right)
        }),
        remainder:,
      ))
  }
}

fn term(input: String) -> option.Option(parser.MatchResult(ASTNode)) {
  // parenthesis | negate | number
  parser.any_of([parenthesis, negate, number])(input)
}

fn parenthesis(input: String) -> option.Option(parser.MatchResult(ASTNode)) {
  // "(" expression ")"
  parser.skip_prefix_and_suffix(string("("), expression, string(")"))(input)
}

fn negate(input: String) -> option.Option(parser.MatchResult(ASTNode)) {
  // "-" expression
  let m =
    parser.skip_prefix(string("-"), expression)
    |> parser.map(fn(x) { Negate(x) })
  m(input)
}

fn number(input: String) -> option.Option(parser.MatchResult(ASTNode)) {
  // https://stackoverflow.com/a/12643073
  // [+-]?([0-9]*[.])?[0-9]+
  let digit = parser.char_range("0", "9")
  let m =
    parser.tuple3(
      parser.any_of([parser.string("+"), parser.string("-")])
        |> parser.option
        |> parser.map(fn(x) { x |> option.unwrap("") }),
      parser.tuple2(
        digit
          |> parser.at_least(0)
          |> parser.map(fn(x) { x |> string.join("") }),
        parser.string("."),
      )
        |> parser.map(fn(x) {
          let #(a, b) = x
          a <> b
        })
        |> parser.option
        |> parser.map(fn(x) { x |> option.unwrap("") }),
      digit |> parser.at_least(1) |> parser.map(fn(x) { x |> string.join("") }),
    )
    |> parser.map(fn(x) {
      let #(a, b, c) = x
      a <> b <> c
    })
    |> skip_whitespace
  case m(input) {
    option.None -> option.None
    option.Some(parser.MatchResult(result:, remainder:)) -> {
      case float.parse(result) {
        Error(_) ->
          case int.parse(result) {
            Error(_) -> option.None
            Ok(result) ->
              option.Some(parser.MatchResult(
                result: Number(int.to_float(result)),
                remainder:,
              ))
          }
        Ok(result) ->
          option.Some(parser.MatchResult(result: Number(result), remainder:))
      }
    }
  }
}

fn string(s: String) -> parser.Matcher(String) {
  skip_whitespace(parser.string(s))
}

fn skip_whitespace(m: parser.Matcher(t)) -> parser.Matcher(t) {
  parser.skip_prefix(
    parser.any_of([
      parser.string(" "),
      parser.string("\t"),
      parser.string("\n"),
      parser.string("\r"),
    ])
      |> parser.at_least(0),
    m,
  )
}
