let char_list_of_string s = List.init (String.length s) (String.get s)
let string_of_char_list l = l |> List.to_seq |> String.of_seq

let string_uncons s =
  match s |> String.to_seq |> Seq.uncons with
  | Some (head, remainder) -> (Some head, remainder |> String.of_seq)
  | None -> (None, "")

let is_whitespace c = c = ' ' || c = '\t' || c = '\n' || c = '\r'

let rec trim_common_whitespace a b =
  match (string_uncons a, string_uncons b) with
  | (Some a_head, a_remainder), (Some b_head, b_remainder)
    when is_whitespace a_head && is_whitespace b_head && a_head = b_head ->
      let this_whitespace = a_head |> String.make 1 in
      let extra_whitespace, a_result, b_result =
        trim_common_whitespace a_remainder b_remainder
      in
      (this_whitespace ^ extra_whitespace, a_result, b_result)
  | _ -> ("", a, b)

let rec trim_common_whitespace_2 inputs =
  let heads, remainders = inputs |> List.map string_uncons |> List.split in
  ()

(* 
let dedent s =
  let lines = String.split_on_char '\n' s in
  s *)
