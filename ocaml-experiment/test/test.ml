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

let trim_common_whitespace_tests =
  "trim common whitespace"
  >::: ([
          (" foo", "  bar", (" ", "foo", " bar"));
          (" \tfoo", " \t bar", (" \t", "foo", " bar"));
          (" \tfoo", "  \tbar", (" ", "\tfoo", " \tbar"));
        ]
       |> List.map (fun (input1, input2, expected) ->
              let ( expected_whitespace,
                    expected_remainder_1,
                    expected_remainder_2 ) =
                expected
              in
              let name =
                Format.sprintf "\"%s\" \"%s\" -> (\"%s\", \"%s\", \"%s\")"
                  input1 input2 expected_whitespace expected_remainder_1
                  expected_remainder_2
              in
              name >:: fun _ ->
              let actual_whitespace, actual_remainder_1, actual_remainder_2 =
                trim_common_whitespace input1 input2
              in
              assert_equal expected_whitespace actual_whitespace
                ~printer:(fun x -> x);
              assert_equal expected_remainder_1 actual_remainder_1
                ~printer:(fun x -> x);
              assert_equal expected_remainder_2 actual_remainder_2
                ~printer:(fun x -> x)))

(* 
let dedent_tests =
  "dedent"
  >::: ([ ("  foo\n   bar", "foo\n bar") (* TODO more tests *) ]
       |> List.map (fun (a, b) ->
              let name = Format.sprintf "%s -> %s" a b in
              name >:: fun _ -> assert_equal b (dedent a))) *)

let tests = "tests" >::: [ is_whitespace_tests; trim_common_whitespace_tests ]
let _ = run_test_tt_main tests
