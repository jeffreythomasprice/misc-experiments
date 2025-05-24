module Parser.Tests

open NUnit.Framework

let LiteralData =
    [ "foo", "foobar", Ok("foo", "bar"); "foo", "fobar", Error "fobar" ]
    |> List.map (fun (parser, input, expected) -> TestCaseData(parser, input, expected))

[<TestCaseSource(nameof LiteralData)>]
let Literal literal input expected =
    let parser = Parsers.Literal literal
    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "literal=%s, input=%s, expected=%A, result=%A" literal input expected result
    )

[<Test>]
let SeqSuccess () =
    let parser = Parsers.Seq [ Parsers.Literal "foo"; Parsers.Literal "bar" ]
    let input = "foobarbaz"
    let expected: Result<(string list * string), string> = Ok([ "foo"; "bar" ], "baz")
    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

[<Test>]
let SeqFailureOnFirst () =
    let parser = Parsers.Seq [ Parsers.Literal "foo"; Parsers.Literal "bar" ]
    let input = "fobarbaz"
    let expected: Result<(string list * string), string> = Error "fobarbaz"
    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

[<Test>]
let SeqFailureOnSecond () =
    let parser = Parsers.Seq [ Parsers.Literal "foo"; Parsers.Literal "bar" ]
    let input = "foobabaz"
    let expected: Result<(string list * string), string> = Error "foobabaz"
    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

[<Test>]
let Seq2Success () =
    let parser = Parsers.Seq2 (Parsers.Literal "foo") (Parsers.Literal "bar")
    let input = "foobarbaz"

    let expected: Result<((string * string) * string), string> =
        Ok(("foo", "bar"), "baz")

    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

[<Test>]
let Seq2FailureOnFirst () =
    let parser = Parsers.Seq2 (Parsers.Literal "foo") (Parsers.Literal "bar")
    let input = "fobarbaz"
    let expected: Result<((string * string) * string), string> = Error "fobarbaz"
    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

[<Test>]
let Seq2FailureOnSecond () =
    let parser = Parsers.Seq2 (Parsers.Literal "foo") (Parsers.Literal "bar")
    let input = "foobabaz"
    let expected: Result<((string * string) * string), string> = Error "foobabaz"
    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

let Seq3Data =
    [ "foobarbazasdf", Ok(("foo", "bar", "baz"), "asdf")
      "fobarbazasdf", Error "fobarbazasdf"
      "foobrbazasdf", Error "foobrbazasdf"
      "foobarbzasdf", Error "foobarbzasdf" ]
    |> List.map (fun (input, expected) -> TestCaseData(input, expected))

[<TestCaseSource(nameof Seq3Data)>]
let Seq3 input expected =
    let parser =
        Parsers.Seq3 (Parsers.Literal "foo") (Parsers.Literal "bar") (Parsers.Literal "baz")

    let result = parser input

    Assert.That(
        result,
        Is.EqualTo expected,
        sprintf "parser=%A, input=%s, expected=%A, result=%A" parser input expected result
    )

// TODO test for Map
// TODO test for Optional
