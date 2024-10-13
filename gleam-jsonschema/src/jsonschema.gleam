import gleam/dict.{type Dict}
import gleam/dynamic
import gleam/int
import gleam/io
import gleam/json
import gleam/list
import gleam/option.{type Option, Some}
import gleam/result
import gleam/string
import gleam/uri

pub type Options {
  Options(resolve_ref: fn(uri.Uri) -> Result(String, Nil))
}

pub type Schema {
  Object(
    common: Common,
    properties: Option(Dict(String, Schema)),
    required: Option(List(String)),
  )
  Array(common: Common, items: Option(Schema))
  Number(common: Common, minimum: Option(Float), maximum: Option(Float))
  Integer(common: Common, minimum: Option(Int), maximum: Option(Int))
  String(common: Common)
}

pub type Common {
  Common(
    typ: Option(String),
    id: Option(String),
    anchor: Option(String),
    schema: Option(String),
    ref: Option(String),
    defs: Option(Dict(String, Schema)),
    title: Option(String),
    description: Option(String),
  )
}

pub fn parse(
  input: dynamic.Dynamic,
  options: Options,
) -> Result(Schema, dynamic.DecodeErrors) {
  use common <- result.try(
    input
    |> dynamic.decode8(
      Common,
      dynamic.optional_field("type", dynamic.string),
      dynamic.optional_field("$id", dynamic.string),
      dynamic.optional_field("$anchor", dynamic.string),
      dynamic.optional_field("$schema", dynamic.string),
      dynamic.optional_field("$ref", dynamic.string),
      dynamic.optional_field(
        "$defs",
        dynamic.dict(dynamic.string, parse(_, options)),
      ),
      dynamic.optional_field("title", dynamic.string),
      dynamic.optional_field("description", dynamic.string),
    ),
  )

  case common.typ, common.ref {
    Some("object"), _ -> parse_object(input, common, options)
    Some("array"), _ -> parse_array(input, common, options)
    Some("number"), _ -> parse_number(input, common)
    Some("integer"), _ -> parse_integer(input, common)
    Some("string"), _ -> parse_string(input, common)
    _, Some(ref) -> resolve_ref(ref, common, options)
    _, _ ->
      Error([
        dynamic.DecodeError(
          expected: "type",
          found: common.typ |> option.unwrap("<none>"),
          path: [],
        ),
      ])
  }
}

fn resolve_ref(
  ref: String,
  common: Common,
  options: Options,
) -> Result(Schema, dynamic.DecodeErrors) {
  use resolved_ref <- result.try(case uri.parse(ref) {
    Ok(uri_ref) -> {
      io.println("TODO ref = " <> ref)
      io.println("TODO fragment = " <> string.inspect(uri_ref.fragment))

      // TODO handle relative refs
      // - we only have a url with no fragment -> pass on to options
      // - we only have a fragment with no other parts -> look for a $defs with that id or anchor in this schema
      // - we have a url with both parts -> resolve via options, then look in that $defs

      options.resolve_ref(uri_ref)
      |> result.map_error(fn(_) {
        [dynamic.DecodeError("resolvable ref", ref, [])]
      })
    }
    Error(_) -> Error([dynamic.DecodeError("valid uri in ref", ref, [])])
  })
  json.decode(resolved_ref, parse(_, options))
  |> result.map_error(json_decode_error_to_dynamic_decode_errors)
}

fn json_decode_error_to_dynamic_decode_errors(
  e: json.DecodeError,
) -> dynamic.DecodeErrors {
  case e {
    json.UnexpectedByte(e) -> [dynamic.DecodeError("valid json", e, [])]
    json.UnexpectedEndOfInput -> [
      dynamic.DecodeError("valid json", "end of input", []),
    ]
    json.UnexpectedFormat(e) -> e
    json.UnexpectedSequence(e) -> [dynamic.DecodeError("valid json", e, [])]
  }
}

fn parse_object(
  input: dynamic.Dynamic,
  common: Common,
  options: Options,
) -> Result(Schema, dynamic.DecodeErrors) {
  input
  |> dynamic.decode2(
    fn(a, b) { Object(common, a, b) },
    dynamic.optional_field(
      "properties",
      dynamic.dict(dynamic.string, fn(input) { parse(input, options) }),
    ),
    dynamic.optional_field("required", dynamic.list(dynamic.string)),
  )
}

fn parse_array(
  input: dynamic.Dynamic,
  common: Common,
  options: Options,
) -> Result(Schema, dynamic.DecodeErrors) {
  input
  |> dynamic.decode1(
    fn(a) { Array(common, a) },
    dynamic.optional_field("items", parse(_, options)),
  )
}

fn parse_number(
  input: dynamic.Dynamic,
  common: Common,
) -> Result(Schema, dynamic.DecodeErrors) {
  input
  |> dynamic.decode2(
    fn(a, b) { Number(common, a, b) },
    dynamic.optional_field("minimum", parse_int_or_float),
    dynamic.optional_field("maximum", parse_int_or_float),
  )
}

fn parse_integer(
  input: dynamic.Dynamic,
  common: Common,
) -> Result(Schema, dynamic.DecodeErrors) {
  input
  |> dynamic.decode2(
    fn(a, b) { Integer(common, a, b) },
    dynamic.optional_field("minimum", dynamic.int),
    dynamic.optional_field("maximum", dynamic.int),
  )
}

fn parse_string(
  input: dynamic.Dynamic,
  common: Common,
) -> Result(Schema, dynamic.DecodeErrors) {
  Ok(String(common))
}

fn parse_int_or_float(
  input: dynamic.Dynamic,
) -> Result(Float, dynamic.DecodeErrors) {
  case dynamic.float(input), dynamic.int(input) {
    Ok(result), _ -> Ok(result)
    _, Ok(result) -> Ok(int.to_float(result))
    Error(e1), Error(e2) -> Error(list.concat([e1, e2]))
  }
}
