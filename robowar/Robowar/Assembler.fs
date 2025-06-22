module Robowar.Assembler

open System
open System.Text.RegularExpressions
open System.Globalization
open Parsers

let identifier: Matcher<string> =
    RegularExpression(new Regex "[a-zA-Z_][a-zA-Z0-9_]*")

let optionalMatcher m f description =
    m
    |> Map(fun s ->
        match f s with
        | true, result -> Ok result
        | _ -> Error(sprintf "failed to parse as %s: %s" description s))

type Number =
    | Int8 of sbyte
    | UInt8 of byte
    | Int16 of int16
    | UInt16 of uint16
    | Int32 of int32
    | UInt32 of uint32
    | Int64 of int64
    | UInt64 of uint64

// TODO all the number matchers should accept an optional suffix
// let x = UInt32.TryParse("af",NumberStyles.HexNumber,CultureInfo.CurrentCulture)

let numberInt8 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) SByte.TryParse "int8"
    |> Map(fun x -> Ok(Int8 x))

let numberUInt8 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) Byte.TryParse "uint8"
    |> Map(fun x -> Ok(UInt8 x))

let numberInt16 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) Int16.TryParse "int16"
    |> Map(fun x -> Ok(Int16 x))

let numberUInt16 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) UInt16.TryParse "uint16"
    |> Map(fun x -> Ok(UInt16 x))

let numberInt32 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) Int32.TryParse "int32"
    |> Map(fun x -> Ok(Int32 x))

let numberUInt32 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) UInt32.TryParse "uint32"
    |> Map(fun x -> Ok(UInt32 x))

let numberInt64 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) Int64.TryParse "int64"
    |> Map(fun x -> Ok(Int64 x))

let numberUInt64 =
    optionalMatcher (RegularExpression(new Regex "\\-?[0-9]+")) UInt64.TryParse "uint64"
    |> Map(fun x -> Ok(UInt64 x))

(*
hexInt32
octalInt32
binaryInt32
anyInt32
... same for 8, 16, 64
number

argument = identifier | number
argumentList = argument ("," argument)*
label = identifier ":"
instruction = label? identifier argumentList?
*)
