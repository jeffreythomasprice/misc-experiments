import gleam/bool
import gleam/dynamic
import gleam/dynamic/decode
import gleam/io
import gleam/json
import gleam/list
import gleam/option
import gleam/string
import simplifile

type JsonSchemaPrimitiveType {
  Null
  Boolean
  Number
  Integer
  String
  Object
  Array
}

fn json_schema_primitive_type_decoder() -> decode.Decoder(
  JsonSchemaPrimitiveType,
) {
  use variant <- decode.then(decode.string)
  case variant {
    "null" -> decode.success(Null)
    "boolean" -> decode.success(Boolean)
    "number" -> decode.success(Number)
    "integer" -> decode.success(Integer)
    "string" -> decode.success(String)
    "object" -> decode.success(Object)
    "array" -> decode.success(Array)
    _ -> decode.failure(Null, "JsonSchemaPrimitiveType")
  }
}

type JsonSchemaType {
  Primitive(JsonSchemaPrimitiveType)
  List(List(JsonSchemaPrimitiveType))
}

fn json_schema_type_decoder() -> decode.Decoder(JsonSchemaType) {
  decode.one_of(
    {
      use result <- decode.then(
        decode.list(json_schema_primitive_type_decoder()),
      )
      decode.success(List(result))
    },
    [
      {
        use result <- decode.then(json_schema_primitive_type_decoder())
        decode.success(Primitive(result))
      },
    ],
  )
}

type JsonSchema {
  JsonSchema(schema: option.Option(String), typ: option.Option(JsonSchemaType))
}

fn json_schema_decoder() -> decode.Decoder(JsonSchema) {
  use schema <- decode.field("$schema", decode.optional(decode.string))
  use typ <- decode.field("type", decode.optional(json_schema_type_decoder()))
  decode.success(JsonSchema(schema:, typ:))
}

type Test {
  Test(description: String, data: dynamic.Dynamic, valid: Bool)
}

fn test_decoder() -> decode.Decoder(Test) {
  use description <- decode.field("description", decode.string)
  use data <- decode.field("data", decode.dynamic)
  use valid <- decode.field("valid", decode.bool)
  decode.success(Test(description:, data:, valid:))
}

type TestCase {
  TestCase(description: String, schema: JsonSchema, tests: List(Test))
}

fn test_case_decoder() -> decode.Decoder(TestCase) {
  use description <- decode.field("description", decode.string)
  use schema <- decode.field("schema", json_schema_decoder())
  use tests <- decode.field("tests", decode.list(test_decoder()))
  decode.success(TestCase(description:, schema:, tests:))
}

pub fn main() -> Nil {
  io.println("Hello from json_schema!")
  let assert Ok(file) =
    simplifile.read(
      from: "submodules/JSON-Schema-Test-Suite/tests/draft2020-12/type.json",
    )

  let assert Ok(test_cases) = json.parse(file, decode.list(test_case_decoder()))
  test_cases
  |> list.each(fn(test_case) {
    let TestCase(description:, schema:, tests:) = test_case
    io.println("test case: " <> description)
    io.println("schema: " <> string.inspect(schema))
    io.println("")
    tests
    |> list.each(fn(t) {
      let Test(description:, data:, valid:) = t
      io.println("    test: " <> description)
      io.println("    data: " <> string.inspect(data))
      io.println("    valid: " <> bool.to_string(valid))
      io.println("")
    })
  })

  Nil
}
