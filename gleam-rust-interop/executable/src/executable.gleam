import gleam/io

@external(erlang, "rust_lib", "add_ints")
pub fn add_ints(a: Int, b: Int) -> Int

pub fn main() -> Nil {
  io.println("Hello from executable!")
  echo add_ints(1, 2) as "add_ints()"
  Nil
}
