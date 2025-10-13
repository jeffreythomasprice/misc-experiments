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

let match_literal_tests =
  "match literal"
  >::: [
         ( "default case sensitivity, matches" >:: fun _ ->
           let m = match_literal "Foo" in
           let actual = m "Foobar" in
           let expected = Some { result = "Foo"; remainder = "bar" } in
           assert_equal expected actual );
         ( "default case sensitivity, does not match" >:: fun _ ->
           let m = match_literal "Foo" in
           let actual = m "foobar" in
           let expected = None in
           assert_equal expected actual );
         ( "case sensitive, matches" >:: fun _ ->
           let m = match_literal "Foo" ~case_sensitive:true in
           let actual = m "Foobar" in
           let expected = Some { result = "Foo"; remainder = "bar" } in
           assert_equal expected actual );
         ( "case sensitive, does not match" >:: fun _ ->
           let m = match_literal "Foo" ~case_sensitive:true in
           let actual = m "foobar" in
           let expected = None in
           assert_equal expected actual );
         ( "case insensitive, matches" >:: fun _ ->
           let m = match_literal "Foo" ~case_sensitive:false in
           let actual = m "Foobar" in
           let expected = Some { result = "Foo"; remainder = "bar" } in
           assert_equal expected actual );
         ( "case insensitive, does not match" >:: fun _ ->
           let m = match_literal "Foo" ~case_sensitive:false in
           let actual = m "FOObar" in
           let expected = Some { result = "FOO"; remainder = "bar" } in
           assert_equal expected actual );
       ]

let match_char_range_tests =
  "match char range"
  >::: [
         ( "a..z, matches" >:: fun _ ->
           let m = match_char_range 'a' 'z' in
           let actual = m "foo" in
           let expected = Some { result = 'f'; remainder = "oo" } in
           assert_equal expected actual );
         ( "a..z, does not match" >:: fun _ ->
           let m = match_char_range 'a' 'z' in
           let actual = m "123" in
           let expected = None in
           assert_equal expected actual );
       ]

let match_seq2_tests =
  "match seq2"
  >::: [
         ( "both match" >:: fun _ ->
           let m = match_seq2 (match_literal "foo") (match_literal "bar") in
           let actual = m "foobarbaz" in
           let expected = Some { result = ("foo", "bar"); remainder = "baz" } in
           assert_equal expected actual );
         ( "first fails" >:: fun _ ->
           let m = match_seq2 (match_literal "foo") (match_literal "bar") in
           let actual = m "fobarbaz" in
           let expected = None in
           assert_equal expected actual );
         ( "second fails" >:: fun _ ->
           let m = match_seq2 (match_literal "foo") (match_literal "bar") in
           let actual = m "foobabaz" in
           let expected = None in
           assert_equal expected actual );
       ]

let tests =
  "tests"
  >::: [
         is_whitespace_tests;
         dedent_tests;
         match_literal_tests;
         match_char_range_tests;
         match_seq2_tests;
       ]

let _ = run_test_tt_main tests
