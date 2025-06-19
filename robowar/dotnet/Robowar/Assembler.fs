module Robowar.Assembler

open System
open System.Text.RegularExpressions
open Parsers

(*
TODO assembler

instruction = identifier argumentList?
argumentList = argument ("," argument)*
argument = identifier | number
identifier = [a-zA-Z_][a-zA-Z0-9_]*
number = ???
*)

let identifier: Matcher<string> =
    RegularExpression(new Regex "[a-zA-Z_][a-zA-Z0-9_]*")

let decimalInt32: Matcher<Int32> =
    RegularExpression(new Regex "\\-?[0-9]+")
    |> Map(fun s ->
        match Int32.TryParse s with
        | true, result -> Ok result
        | _ -> Error(sprintf "failed to parse as int32: %s" s))
