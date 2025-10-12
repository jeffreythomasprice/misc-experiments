open OUnit2
open Lib

let is_whitespace_tests =
  "is whitespace"
  >::: ([ (' ', true); ('\t', true); ('\n', true); ('\r', true); ('a', false) ]
       |> List.map (fun (input, expected) ->
              let name =
                Format.sprintf "%c (%i) -> %b" input (int_of_char input)
                  expected
              in
              name >:: fun _ ->
              assert_equal expected (is_whitespace input)
                ~printer:string_of_bool))

let trim_common_leading_whitespace_tests =
  "trim common whitespace"
  >::: ([
          ( "has common whitespace",
            [ " foo"; "  bar"; "   baz" ],
            [ "foo"; " bar"; "  baz" ] );
          ( "has common whitespace, 2",
            [ "\t  \tfoo"; "\t bar"; "\t \t\t\tbaz" ],
            [ " \tfoo"; "bar"; "\t\t\tbaz" ] );
          ( "no common prefix",
            [ "\t  \tfoo"; "bar"; " baz" ],
            [ "\t  \tfoo"; "bar"; " baz" ] );
          ( "non-whitespace common prefix",
            [ "aaafoo"; "aaabar"; "aaabaz" ],
            [ "aaafoo"; "aaabar"; "aaabaz" ] );
        ]
       |> List.map (fun (name, inputs, expected) ->
              name >:: fun _ ->
              let actual = trim_common_leading_whitespace inputs in
              assert_equal expected actual))

(* TODO dedent tests *)

let tests =
  "tests" >::: [ is_whitespace_tests; trim_common_leading_whitespace_tests ]

let _ = run_test_tt_main tests
