import gleam/io

@external(erlang, "rust_lib", "foo")
pub fn foo() -> a

pub fn main() -> Nil {
  io.println("Hello from executable!")
  echo foo() as "foo"
}
