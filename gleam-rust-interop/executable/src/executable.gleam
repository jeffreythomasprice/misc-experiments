import gleam/io

@external(erlang, "rust_lib", "add_ints")
fn add_ints(a: Int, b: Int) -> Int

type Data

@external(erlang, "rust_lib", "new_data")
fn new_data(initial_count: Int) -> Data

@external(erlang, "rust_lib", "data_get")
fn data_get(data: Data) -> Int

@external(erlang, "rust_lib", "data_increment")
fn data_increment(data: Data, increment: Int) -> Nil

pub fn main() -> Nil {
  io.println("Hello from executable!")
  echo add_ints(1, 2) as "add_ints()"

  let data = new_data(1)
  echo data as "data"

  echo data_get(data) as "data_get"

  data_increment(data, 1)
  echo data_get(data) as "data_get"

  data_increment(data, 40)
  echo data_get(data) as "data_get"

  Nil
}
