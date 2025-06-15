module Robowar.Program

open Robowar.Parsers

// TODO clean up main

let unwrap result =
    match result with
    | Ok result -> result
    | Error e -> failwith (sprintf "result was error: %A" e)

let main () =
    let input = InputString.FromString "Hello, World!"
    printfn "input = %A" input

    let matcher = LiteralString "Hello, "

    let result = matcher input |> unwrap
    printfn "result = %A" result

    let matcher =
        RegularExpression(new System.Text.RegularExpressions.Regex("[a-zA-Z]+"))

    let result = matcher result.Remainder |> unwrap
    printfn "result = %A" result

    ()

main ()
