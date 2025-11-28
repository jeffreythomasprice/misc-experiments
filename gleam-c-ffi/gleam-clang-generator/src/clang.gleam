import gleam/dynamic/decode
import gleam/int
import gleam/io
import gleam/json
import gleam/option
import gleam/regexp
import gleam/result
import gleam/string
import gleamyshell

pub type Loc {
  Loc(
    offset: option.Option(Int),
    file: option.Option(String),
    line: option.Option(Int),
    col: option.Option(Int),
    tok_len: option.Option(Int),
  )
}

fn loc_decoder() -> decode.Decoder(Loc) {
  use offset <- decode.optional_field(
    "offset",
    option.None,
    decode.optional(decode.int),
  )
  use file <- decode.optional_field(
    "file",
    option.None,
    decode.optional(decode.string),
  )
  use line <- decode.optional_field(
    "line",
    option.None,
    decode.optional(decode.int),
  )
  use col <- decode.optional_field(
    "col",
    option.None,
    decode.optional(decode.int),
  )
  use tok_len <- decode.optional_field(
    "tokLen",
    option.None,
    decode.optional(decode.int),
  )
  decode.success(Loc(offset:, file:, line:, col:, tok_len:))
}

pub type Type {
  Type(qual_type: option.Option(String))
}

fn type_decoder() -> decode.Decoder(Type) {
  use qual_type <- decode.optional_field(
    "qualType",
    option.None,
    decode.optional(decode.string),
  )
  decode.success(Type(qual_type:))
}

pub type AST {
  AST(
    id: String,
    kind: String,
    loc: option.Option(Loc),
    // TODO range
    is_implicit: option.Option(Bool),
    name: option.Option(String),
    typ: option.Option(Type),
    inner: List(AST),
    function_return_type: option.Option(String),
  )
}

fn clang_ast_decoder() -> decode.Decoder(AST) {
  use id <- decode.field("id", decode.string)
  use kind <- decode.field("kind", decode.string)
  use loc <- decode.optional_field(
    "loc",
    option.None,
    decode.optional(loc_decoder()),
  )
  // TODO range
  use is_implicit <- decode.optional_field(
    "isImplicit",
    option.None,
    decode.optional(decode.bool),
  )
  use name <- decode.optional_field(
    "name",
    option.None,
    decode.optional(decode.string),
  )
  use typ <- decode.optional_field(
    "type",
    option.None,
    decode.optional(type_decoder()),
  )
  use inner <- decode.optional_field(
    "inner",
    [],
    decode.list(clang_ast_decoder()),
  )

  let function_return_type = case kind, typ {
    "FunctionDecl", option.Some(Type(qual_type: option.Some(qual_type))) -> {
      let assert Ok(r) =
        regexp.compile(
          "^([^\\(\\)]+)\\s+\\(.*$",
          with: regexp.Options(case_insensitive: False, multi_line: False),
        )
      case regexp.scan(with: r, content: qual_type) {
        [regexp.Match(submatches: [option.Some(result)], ..)] ->
          option.Some(result)
        _ -> option.None
      }
    }
    _, _ -> option.None
  }

  decode.success(AST(
    id:,
    kind:,
    loc:,
    is_implicit:,
    name:,
    typ:,
    inner:,
    function_return_type:,
  ))
}

pub fn parse_clang_ast_file(path: String) -> Result(AST, String) {
  use ast_str <- result.try(
    case
      gleamyshell.execute("clang", in: ".", args: [
        "-Xclang",
        "-ast-dump=json",
        "-fsyntax-only",
        path,
      ])
    {
      Ok(gleamyshell.CommandOutput(exit_code: 0, output:)) -> {
        Ok(output)
      }
      Ok(gleamyshell.CommandOutput(exit_code:, output:)) -> {
        Error(
          "command exited with code: "
          <> int.to_string(exit_code)
          <> ", output:\n"
          <> output,
        )
      }
      Error(e) -> Error("error running command: " <> e)
    },
  )

  io.println("TODO ast_str: " <> ast_str)

  use result <- result.try(
    json.parse(ast_str, clang_ast_decoder())
    |> result.map_error(fn(e) { string.inspect(e) }),
  )

  Ok(result)
}
