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
