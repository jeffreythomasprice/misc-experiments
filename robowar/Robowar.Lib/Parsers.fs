namespace Robowar.Lib

module Parsers =
    type Parser<'T> = string -> Result<'T * string, string>

    let Map (p: Parser<'T>) (f: 'T -> 'R) (input: string) =
        input |> p |> Result.bind (fun (result, remainder) -> Ok(f result, remainder))

    let Optional (p: Parser<'T>) (input: string) : Result<'T option * string, string> =
        match p input with
        | Ok(result, remainder) -> Ok(Some result, remainder)
        | Error _ -> Ok(None, input)


    let Literal (literal: string) (input: string) =
        if input.StartsWith(literal) then
            Ok(literal, input.Substring literal.Length)
        else
            Error input

    let rec Seq<'T> (parsers: (Parser<'T>) list) (input: string) =
        let originalInput = input

        match parsers with
        | [] -> Ok([], input)
        | p :: rest ->
            match p input with
            | Ok(firstResult, remainder) ->
                match Seq rest remainder with
                | Ok(restResults, remainder) -> Ok(firstResult :: restResults, remainder)
                | Error _ -> Error originalInput
            | Error _ -> Error originalInput

    let Seq2 p1 p2 input =
        let originalInput = input

        match p1 input with
        | Ok(r1, remainder) ->
            match p2 remainder with
            | Ok(r2, remainder) -> Ok((r1, r2), remainder)
            | Error _ -> Error originalInput
        | Error _ -> Error originalInput

    let Seq3 p1 p2 p3 input =
        let originalInput = input

        match Seq2 p1 p2 input with
        | Ok((r1, r2), remainder) ->
            match p3 remainder with
            | Ok(r3, remainder) -> Ok((r1, r2, r3), remainder)
            | Error _ -> Error originalInput
        | Error _ -> Error originalInput
