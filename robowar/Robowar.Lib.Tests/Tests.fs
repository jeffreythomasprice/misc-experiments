module Tests

open System
open Xunit
open Robowar.Lib.Parsers

let ``map test data``: obj[] list =
    [ [| (Literal "foo") |> Map(fun s -> s.Length)
         "foobar"
         Result<int * string, string>.Ok(3, "bar") |]
      // TODO failure
      ]

[<Theory>]
[<MemberData(nameof ``map test data``)>]
let ``map`` (parser: Parser<int>) input expected =
    let actual = parser input
    Assert.True(expected.Equals(actual), sprintf "%A != %A" expected actual)

// TODO optional

let ``literal test data``: obj[] list =
    [ [| Literal "foo"; "foobar"; Result<string * string, string>.Ok("foo", "bar") |]
      [| Literal "foo"; "fobar"; Result<string * string, string>.Error("fobar") |] ]

[<Theory>]
[<MemberData(nameof ``literal test data``)>]
let ``literal`` (parser: Parser<string>) input expected =
    let actual = parser input
    Assert.True(expected.Equals(actual), sprintf "%A != %A" expected actual)

let ``seq test data``: obj[] list =
    [ [| Seq [ Literal "foo"; Literal "bar" ]
         "foobarbaz"
         Result<(string list * string), string>.Ok([ "foo"; "bar" ], "baz") |]
      [| Seq [ Literal "foo"; Literal "bar" ]
         "fobarbaz"
         Result<(string list * string), string>.Error "fobarbaz" |]
      [| Seq [ Literal "foo"; Literal "bar" ]
         "foobabaz"
         Result<(string list * string), string>.Error "foobabaz" |] ]

[<Theory>]
[<MemberData(nameof ``seq test data``)>]
let ``seq`` (parser: Parser<string list>) input expected =
    let actual = parser input
    Assert.True(expected.Equals(actual), sprintf "%A != %A" expected actual)

let ``seq2 test data``: obj[] list =
    [ [| Seq2 (Literal "foo") (Literal "bar")
         "foobarbaz"
         Result<((string * string) * string), string>.Ok(("foo", "bar"), "baz") |]
      [| Seq2 (Literal "foo") (Literal "bar")
         "fobarbaz"
         Result<((string * string) * string), string>.Error("fobarbaz") |]
      [| Seq2 (Literal "foo") (Literal "bar")
         "foobabaz"
         Result<((string * string) * string), string>.Error("foobabaz") |] ]

[<Theory>]
[<MemberData(nameof ``seq2 test data``)>]
let ``seq2`` (parser: Parser<string * string>) input expected =
    let actual = parser input
    Assert.True(expected.Equals(actual), sprintf "%A != %A" expected actual)

let ``seq3 test data``: obj[] list =
    [ [| Seq3 (Literal "foo") (Literal "bar") (Literal "baz")
         "foobarbazasdf"
         Result<((string * string * string) * string), string>.Ok(("foo", "bar", "baz"), "asdf") |]
      [| Seq3 (Literal "foo") (Literal "bar") (Literal "baz")
         "fobarbazasdf"
         Result<((string * string * string) * string), string>.Error("fobarbazasdf") |]
      [| Seq3 (Literal "foo") (Literal "bar") (Literal "baz")
         "foobabazasdf"
         Result<((string * string * string) * string), string>.Error("foobabazasdf") |]
      [| Seq3 (Literal "foo") (Literal "bar") (Literal "baz")
         "foobarbaasdf"
         Result<((string * string * string) * string), string>.Error("foobarbaasdf") |] ]

[<Theory>]
[<MemberData(nameof ``seq3 test data``)>]
let ``seq3`` (parser: Parser<string * string * string>) input expected =
    let actual = parser input
    Assert.True(expected.Equals(actual), sprintf "%A != %A" expected actual)
