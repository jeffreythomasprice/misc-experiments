open OUnit2

let tests = "tests" >::: [ Strings.tests; Parsers.tests ]
let _ = run_test_tt_main tests
