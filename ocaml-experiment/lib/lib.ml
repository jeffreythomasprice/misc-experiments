let char_list_of_string s = List.init (String.length s) (String.get s)
let string_of_char_list l = l |> List.to_seq |> String.of_seq

let string_uncons s =
  match s |> String.to_seq |> Seq.uncons with
  | Some (head, remainder) -> (Some head, remainder |> String.of_seq)
  | None -> (None, "")

let is_whitespace c = c = ' ' || c = '\t' || c = '\n' || c = '\r'

let rec all_elements_equal inputs =
  match inputs with
  | [] -> true
  | [ _ ] -> true
  | a :: b :: c -> a = b && all_elements_equal (b :: c)

let rec trim_common_leading_whitespace inputs =
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
  | _, true, true -> trim_common_leading_whitespace remainders
  (* no inputs, mismatched first character, etc. *)
  | _ -> inputs
