import gleam/erlang/process
import gleam/int
import gleam/io
import gleam/otp/actor

@external(erlang, "rust_lib", "add_ints")
fn add_ints(a: Int, b: Int) -> Int

type Data

@external(erlang, "rust_lib", "new_data")
fn new_data(initial_count: Int) -> Data

@external(erlang, "rust_lib", "data_get")
fn data_get(data: Data) -> Int

@external(erlang, "rust_lib", "data_increment")
fn data_increment(data: Data, increment: Int) -> Nil

type Message {
  Increment(Int)
  Get(process.Subject(Int))
}

pub fn main() -> Nil {
  echo add_ints(1, 2) as "add_ints()"

  let data = new_data(1)
  let assert Ok(actor) =
    actor.new(data)
    |> actor.on_message(fn(data, msg) {
      case msg {
        Increment(amount) -> {
          io.println("calling increment, amount: " <> int.to_string(amount))
          data_increment(data, amount)
          Nil
        }
        Get(subject) -> {
          let result = data_get(data)
          io.println("calling get, result: " <> int.to_string(result))
          process.send(subject, result)
          Nil
        }
      }
      actor.continue(data)
    })
    |> actor.start

  io.println(
    "from main, get result: " <> int.to_string(actor.call(actor.data, 10, Get)),
  )
  actor.send(actor.data, Increment(1))
  io.println(
    "from main, get result: " <> int.to_string(actor.call(actor.data, 10, Get)),
  )
  actor.send(actor.data, Increment(40))
  io.println(
    "from main, get result: " <> int.to_string(actor.call(actor.data, 10, Get)),
  )

  Nil
}
