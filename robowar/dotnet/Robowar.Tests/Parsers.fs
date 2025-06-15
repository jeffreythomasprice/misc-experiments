module Tests

open System
open System.Text.RegularExpressions
open Xunit
open Robowar.Parsers

let commonTest<'T when 'T: equality> (matcher: Matcher<'T>) (input: InputString) (expected: MatchResult<'T>) =
    let actual = matcher input
    let message = sprintf "actual != expected\nactual: %A\nexpected: %A" actual expected
    Assert.True(actual.Equals expected, message)

let ``literal string data``: obj array list =
    [ [| LiteralString "foo"
         InputString.FromString "foo bar"
         MatchResult<string>.Ok
             { Result = "foo"
               Remainder =
                 { Input = " bar"
                   Location = { Line = 0; Column = 3 } } } |]
      [| LiteralString "foo"
         InputString.FromString "bar foo"
         MatchResult<string>.Error
             { Expected = "foo"
               Remainder =
                 { Input = "bar foo"
                   Location = { Line = 0; Column = 0 } } } |] ]

[<Theory>]
[<MemberData(nameof (``literal string data``))>]
let ``literal string test`` matcher input expected = commonTest matcher input expected

let ``regex data``: obj array list =
    [ [| RegularExpression(new Regex "[0-9]+")
         InputString.FromString "123 foo"
         MatchResult<string>.Ok
             { Result = "123"
               Remainder =
                 { Input = " foo"
                   Location = { Line = 0; Column = 3 } } } |]
      [| RegularExpression(new Regex "[0-9]+")
         InputString.FromString "foo 123"
         MatchResult<string>.Error
             { Expected = "[0-9]+"
               Remainder =
                 { Input = "foo 123"
                   Location = { Line = 0; Column = 0 } } } |] ]

[<Theory>]
[<MemberData(nameof (``regex data``))>]
let ``regex test`` matcher input expected = commonTest matcher input expected

let ``seq2 data``: obj array list =
    [ [| Seq2 (LiteralString "foo") (LiteralString "bar")
         InputString.FromString "foobarbaz"
         MatchResult<string * string>.Ok
             { Result = "foo", "bar"
               Remainder =
                 { Input = "baz"
                   Location = { Line = 0; Column = 6 } } } |]
      [| Seq2 (LiteralString "foo") (LiteralString "bar")
         InputString.FromString "fobarbaz"
         MatchResult<string * string>.Error
             { Expected = "foo"
               Remainder =
                 { Input = "fobarbaz"
                   Location = { Line = 0; Column = 0 } } } |]
      [| Seq2 (LiteralString "foo") (LiteralString "bar")
         InputString.FromString "foobrbaz"
         MatchResult<string * string>.Error
             { Expected = "bar"
               Remainder =
                 { Input = "foobrbaz"
                   Location = { Line = 0; Column = 0 } } } |] ]

[<Theory>]
[<MemberData(nameof (``seq2 data``))>]
let ``seq2 test`` matcher input expected = commonTest matcher input expected

let ``seq3 data``: obj array list =
    [ [| Seq3 (LiteralString "foo") (LiteralString "bar") (LiteralString "baz")
         InputString.FromString "foobarbazasdf"
         MatchResult<string * string * string>.Ok
             { Result = "foo", "bar", "baz"
               Remainder =
                 { Input = "asdf"
                   Location = { Line = 0; Column = 9 } } } |]
      [| Seq3 (LiteralString "foo") (LiteralString "bar") (LiteralString "baz")
         InputString.FromString "fobarbazasdf"
         MatchResult<string * string * string>.Error
             { Expected = "foo"
               Remainder =
                 { Input = "fobarbazasdf"
                   Location = { Line = 0; Column = 0 } } } |]
      [| Seq3 (LiteralString "foo") (LiteralString "bar") (LiteralString "baz")
         InputString.FromString "foobrbazasdf"
         MatchResult<string * string * string>.Error
             { Expected = "bar"
               Remainder =
                 { Input = "foobrbazasdf"
                   Location = { Line = 0; Column = 0 } } } |]
      [| Seq3 (LiteralString "foo") (LiteralString "bar") (LiteralString "baz")
         InputString.FromString "foobarbzasdf"
         MatchResult<string * string * string>.Error
             { Expected = "baz"
               Remainder =
                 { Input = "foobarbzasdf"
                   Location = { Line = 0; Column = 0 } } } |] ]

[<Theory>]
[<MemberData(nameof (``seq3 data``))>]
let ``seq3 test`` matcher input expected = commonTest matcher input expected

let ``oneOf data``: obj array list =
    [ [| OneOf [ LiteralString "foo"; RegularExpression(new Regex "[0-9]+") ]
         InputString.FromString "foo123asdf"
         MatchResult<string>.Ok
             { Result = "foo"
               Remainder =
                 { Input = "123asdf"
                   Location = { Line = 0; Column = 3 } } } |]
      [| OneOf [ LiteralString "foo"; RegularExpression(new Regex "[0-9]+") ]
         InputString.FromString "123fooasdf"
         MatchResult<string>.Ok
             { Result = "123"
               Remainder =
                 { Input = "fooasdf"
                   Location = { Line = 0; Column = 3 } } } |]
      [| OneOf [ LiteralString "foo"; RegularExpression(new Regex "[0-9]+") ]
         InputString.FromString "asdf123foo"
         MatchResult<string>.Error
             { Expected = "one of [\"foo\"; \"[0-9]+\"]"
               Remainder =
                 { Input = "asdf123foo"
                   Location = { Line = 0; Column = 0 } } } |] ]

[<Theory>]
[<MemberData(nameof (``oneOf data``))>]
let ``oneOf test`` matcher input expected = commonTest matcher input expected

(*
TODO repeat tests
*)
