module Robowar.Parsers

open System
open System.Text.RegularExpressions

type Location =
    { Line: int
      Column: int }

    member this.AdvanceChar(c: char) =
        if c = '\n' then
            { Line = this.Line + 1; Column = 0 }
        elif Char.IsControl c then
            this
        else
            { Line = this.Line
              Column = this.Column + 1 }

    member this.AdvanceString(s: string) =
        Seq.fold (fun (result: Location) next -> result.AdvanceChar next) this s

type InputString =
    { Input: string
      Location: Location }

    static member FromString(input: string) =
        { Input = input
          Location = { Line = 0; Column = 0 } }

    member this.Substring(startIndex: int) =
        let remainder = this.Input.Substring startIndex
        let location = this.Location.AdvanceString(this.Input.Substring(0, startIndex))

        { Input = remainder
          Location = location }

type MatchSuccess<'T> = { Result: 'T; Remainder: InputString }

type MatchFailure =
    { Expected: string
      Remainder: InputString }

type MatchResult<'T> = Result<MatchSuccess<'T>, MatchFailure>

type Matcher<'T> = InputString -> MatchResult<'T>

let LiteralString (s: string) : Matcher<string> =
    fun (input: InputString) ->
        if input.Input.StartsWith s then
            Ok
                { Result = s
                  Remainder = input.Substring s.Length }
        else
            Error { Expected = s; Remainder = input }

let RegularExpression (r: Regex) : Matcher<string> =
    fun (input: InputString) ->
        let m = r.Match input.Input

        if m.Success && m.Index = 0 then
            Ok
                { Result = m.Value
                  Remainder = input.Substring m.Value.Length }
        else
            Error
                { Expected = sprintf "%A" r
                  Remainder = input }

let Seq2<'T1, 'T2> (m1: Matcher<'T1>) (m2: Matcher<'T2>) : Matcher<'T1 * 'T2> =
    fun (input: InputString) ->
        match m1 input with
        | Ok { Result = r1; Remainder = remainder } ->
            match m2 remainder with
            | Ok { Result = r2; Remainder = remainder } ->
                Ok
                    { Result = r1, r2
                      Remainder = remainder }
            | Error { Expected = m } -> Error { Expected = m; Remainder = input }
        | Error { Expected = m } -> Error { Expected = m; Remainder = input }

let Seq3<'T1, 'T2, 'T3> (m1: Matcher<'T1>) (m2: Matcher<'T2>) (m3: Matcher<'T3>) : Matcher<'T1 * 'T2 * 'T3> =
    let prefix = Seq2 m1 m2

    fun (input: InputString) ->
        match prefix input with
        | Ok { Result = r1, r2
               Remainder = remainder } ->
            match m3 remainder with
            | Ok { Result = r3; Remainder = remainder } ->
                Ok
                    { Result = r1, r2, r3
                      Remainder = remainder }
            | Error { Expected = m } -> Error { Expected = m; Remainder = input }
        | Error { Expected = m } -> Error { Expected = m; Remainder = input }

let OneOf<'T> (matchers: Matcher<'T> list) : Matcher<'T> =
    let rec f (matchers: Matcher<'T> list) (input: InputString) : Result<MatchSuccess<'T>, string list> =
        match matchers with
        // we have at least one real matcher
        | m :: rest ->
            match m input with
            // success, so we're done
            | Ok r -> Ok r
            | Error { Expected = message } ->
                match f rest input with
                // one of the remaining matchers succeeded, so we can abort
                | Ok r -> Ok r
                // none succeeded so just combine the errors
                | Error restMessages -> Error(message :: restMessages)
        // we need to count this as a failure because if we get here it's because none of the real matchers worked
        | [] -> Error []

    fun (input: InputString) ->
        match f matchers input with
        | Ok r -> Ok r
        | Error messages ->
            Error
                { Expected = sprintf "one of %A" messages
                  Remainder = input }

type RepeatOptions =
    | AtLeast of Min: int
    | AtMost of Max: int
    | Range of Min: int * Max: int

let Repeat<'T> (m: Matcher<'T>) (options: RepeatOptions) : Matcher<'T list> =
    let isValidNumberOfRepetitions x =
        match options with
        | AtLeast min -> x >= min
        | AtMost max -> x <= max
        | Range(min, max) -> x >= min && x <= max

    let rec next results input =
        let curLen = List.length results

        // if one more result would break it we can abort
        if
            isValidNumberOfRepetitions curLen
            && not (isValidNumberOfRepetitions (curLen + 1))
        then
            Ok { Result = results; Remainder = input }
        else
            // try to match another result
            match m input with
            // if we succeed we can append this result to our list and recurse
            | Ok { Result = result
                   Remainder = remainder } -> next (result :: results) remainder
            // if we fail we can abort with either a success or a failure depending on whether the current number of matches is good
            | Error errorValue ->
                if isValidNumberOfRepetitions curLen then
                    Ok { Result = results; Remainder = input }
                else
                    Error errorValue

    fun (input: InputString) ->
        next [] input
        |> Result.map
            (fun
                { Result = results
                  Remainder = remainder } ->
                { Result = List.rev results
                  Remainder = remainder })

(*
TODO optional
*)
