open Format
open Lib;;

print_string "Hello, World!\n"

let double x = x * 2;;

print_int (double 5);;
print_newline ();;
5 |> double |> print_int;;
print_newline ();;
print_string ("foo" ^ " " ^ (42 |> Int.to_string) ^ "\n");;

print_string ({|
  this is a multi-
  line string
|} |> dedent)
