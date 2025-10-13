open OUnit2
open Lib.Parsers

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
         ( "fail on 1" >:: fun _ ->
           let m = match_seq2 (match_literal "foo") (match_literal "bar") in
           let actual = m "fobarbaz" in
           let expected = None in
           assert_equal expected actual );
         ( "fail on 2" >:: fun _ ->
           let m = match_seq2 (match_literal "foo") (match_literal "bar") in
           let actual = m "foobabaz" in
           let expected = None in
           assert_equal expected actual );
       ]

let match_seq3_tests =
  "match seq3"
  >::: [
         ( "both match" >:: fun _ ->
           let m =
             match_seq3 (match_literal "aaa") (match_literal "bbb")
               (match_literal "ccc")
           in
           let actual = m "aaabbbccc___" in
           let expected =
             Some { result = ("aaa", "bbb", "ccc"); remainder = "___" }
           in
           assert_equal expected actual );
         ( "fail on 1" >:: fun _ ->
           let m =
             match_seq3 (match_literal "aaa") (match_literal "bbb")
               (match_literal "ccc")
           in
           let actual = m "aabbbccc___" in
           let expected = None in
           assert_equal expected actual );
         ( "fail on 2" >:: fun _ ->
           let m =
             match_seq3 (match_literal "aaa") (match_literal "bbb")
               (match_literal "ccc")
           in
           let actual = m "aaabbccc___" in
           let expected = None in
           assert_equal expected actual );
         ( "fail on 3" >:: fun _ ->
           let m =
             match_seq3 (match_literal "aaa") (match_literal "bbb")
               (match_literal "ccc")
           in
           let actual = m "aaabbbcc___" in
           let expected = None in
           assert_equal expected actual );
       ]

let tests =
  "parsers"
  >::: [
         match_literal_tests;
         match_char_range_tests;
         match_seq2_tests;
         match_seq3_tests;
       ]
