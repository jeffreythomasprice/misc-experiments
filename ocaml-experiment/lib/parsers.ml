open Strings

type 'a match_result = { result : 'a; remainder : string } [@@deriving show]
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

let rec match_seq (matchers : 'a matcher list) : 'a list matcher =
  match matchers with
  (* base case, no matchers, input passed through unchanged *)
  | [] -> fun input -> Some { result = []; remainder = input }
  | m :: matchers -> (
      (* a matcher that handles the 2nd and on *)
      let match_rest = match_seq matchers in
      fun input ->
        (* apply the first matcher *)
        match m input with
        | Some { result = result_head; remainder } -> (
            (* apply the rest of the matchers *)
            match match_rest remainder with
            | Some { result = result_tail; remainder } ->
                Some { result = result_head :: result_tail; remainder }
            | None -> None)
        | None -> None)

let match_seq2 (m1 : 't1 matcher) (m2 : 't2 matcher) : ('t1 * 't2) matcher =
 fun input ->
  match m1 input with
  | Some { result = r1; remainder } -> (
      match m2 remainder with
      | Some { result = r2; remainder } -> Some { result = (r1, r2); remainder }
      | None -> None)
  | None -> None

let match_seq3 (m1 : 't1 matcher) (m2 : 't2 matcher) (m3 : 't3 matcher) :
    ('t1 * 't2 * 't3) matcher =
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
