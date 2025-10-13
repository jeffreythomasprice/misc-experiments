open Strings

type 'a match_result = { result : 'a; remainder : string }
type 'a matcher = string -> 'a match_result option

let match_literal (s : string) ?(case_sensitive : bool = true) : string matcher
    =
 fun input ->
  let has_prefix =
    if case_sensitive then String.starts_with input ~prefix:s
    else
      String.starts_with
        (String.lowercase_ascii input)
        ~prefix:(String.lowercase_ascii s)
  in
  if has_prefix then
    let result = String.sub input 0 (String.length s) in
    let remainder =
      String.sub input (String.length s) (String.length input - String.length s)
    in
    Some { result; remainder }
  else None

let match_char_range (lower : char) (upper : char) : char matcher =
 fun input ->
  match string_uncons input with
  | Some result, remainder ->
      if result >= lower && result <= upper then Some { result; remainder }
      else None
  | None, _ -> None

let match_seq2 m1 m2 =
 fun input ->
  match m1 input with
  | Some { result = r1; remainder } -> (
      match m2 remainder with
      | Some { result = r2; remainder } -> Some { result = (r1, r2); remainder }
      | None -> None)
  | None -> None

let match_seq3 m1 m2 m3 =
 fun input ->
  match m1 input with
  | Some { result = r1; remainder } -> (
      match m2 remainder with
      | Some { result = r2; remainder } -> (
          match m3 remainder with
          | Some { result = r3; remainder } ->
              Some { result = (r1, r2, r3); remainder }
          | None -> None)
      | None -> None)
  | None -> None

(*
TODO match seq list
TODO match any of
TODO match optional
TODO match in range
TODO match at least zero
TODO match at least one
TODO ignore prefix
TODO ignore suffix
TODO ignore prefix and suffix
TODO match integers
TODO match floats
TODO match any whitespace
*)
