module Robowar.Program

open Robowar.Parsers

// TODO clean up main

let unwrap result =
    match result with
    | Ok result -> result
    | Error e -> failwith (sprintf "result was error: %A" e)

let main () =
    let input = InputString.FromString "123 456 789 foobar"
    printfn "input = %A" input

    let matcher =
        Repeat (RegularExpression(new System.Text.RegularExpressions.Regex "[0-9]+ ")) (AtLeast 1)

    let result = matcher input |> unwrap
    printfn "result = %A" result
    ()

main ()
