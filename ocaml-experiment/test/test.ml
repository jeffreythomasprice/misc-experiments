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

let dedent_tests =
  "dedent"
  >::: ([
          ("has common whitespace", " foo\n  bar\n   baz", "foo\n bar\n  baz");
          ( "has common whitespace, 2",
            "\t  \tfoo\n\t bar\n\t \t\t\tbaz",
            " \tfoo\nbar\n\t\t\tbaz" );
          ("no common prefix", "\t  \tfoo\nbar\n baz", "\t  \tfoo\nbar\n baz");
          ( "non-whitespace common prefix",
            "aaafoo\naaabar\naaabaz",
            "aaafoo\naaabar\naaabaz" );
        ]
       |> List.map (fun (name, input, expected) ->
              name >:: fun _ ->
              let actual = dedent input in
              assert_equal expected actual))

let tests = "tests" >::: [ is_whitespace_tests; dedent_tests ]
let _ = run_test_tt_main tests
