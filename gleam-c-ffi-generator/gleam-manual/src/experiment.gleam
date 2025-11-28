import gleam/erlang/charlist
import gleam/int
import gleam/io

// TODO no hello
@external(erlang, "libexperiment", "hello")
pub fn hello() -> charlist.Charlist

@external(erlang, "libexperiment", "foobar")
pub fn foobar(s: charlist.Charlist, x: Int) -> Int

pub fn main() -> Nil {
  io.println("hello() = " <> charlist.to_string(hello()))

  let result = foobar("42" |> charlist.from_string, 1)
  io.println("foobar(...) = " <> int.to_string(result))

  Nil
}
