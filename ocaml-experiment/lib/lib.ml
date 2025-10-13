let char_list_of_string s = List.init (String.length s) (String.get s)
let string_of_char_list l = l |> List.to_seq |> String.of_seq

let string_uncons s =
  match s |> String.to_seq |> Seq.uncons with
  | Some (head, remainder) -> (Some head, remainder |> String.of_seq)
  | None -> (None, "")

let is_whitespace c = c = ' ' || c = '\t' || c = '\n' || c = '\r'

let dedent s =
  let rec all_elements_equal inputs =
    match inputs with
    | [] -> true
    | [ _ ] -> true
    | a :: b :: c -> a = b && all_elements_equal (b :: c)
  in

  let rec longest_common_whitespace_prefix inputs =
    let heads, remainders = inputs |> List.map string_uncons |> List.split in

    let all_heads_equal = all_elements_equal heads in

    let leading_head_is_whitespace =
      match heads with
      | [] -> false
      | Some head :: _ -> is_whitespace head
      | None :: _ -> false
    in

    match (heads, all_heads_equal, leading_head_is_whitespace) with
    (* the first char is whitespace in all inputs, and it's the same whitespace character *)
    | Some head :: _, true, true ->
        String.make 1 head ^ longest_common_whitespace_prefix remainders
    (* no inputs, mismatched first character, etc. *)
    | _ -> ""
  in

  let trim_prefix s prefix =
    if String.length prefix <= String.length s then
      String.sub s (String.length prefix)
        (String.length s - String.length prefix)
    else s
  in

  let lines = String.split_on_char '\n' s in
  let non_empty_lines =
    lines |> List.filter (fun line -> String.trim line <> "")
  in
  let leading_whitespace = longest_common_whitespace_prefix non_empty_lines in
  let trimmed_lines =
    lines
    |> List.map (fun line ->
           let trimmed = String.trim line in
           if trimmed == "" then "" else trim_prefix line leading_whitespace)
  in
  String.concat "\n" trimmed_lines

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

(*
TODO match seq3
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
