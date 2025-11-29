import gleam/int
import lustre
import lustre/element
import lustre/element/html
import lustre/event

type Model {
  Model(count: Int)
}

type Message {
  Increment
  Decrement
}

pub fn main() -> Nil {
  let app = lustre.simple(init, update, view)
  let assert Ok(_) = lustre.start(app, "#app", Nil)
  Nil
}

fn init(_) -> Model {
  Model(count: 0)
}

fn update(model: Model, msg: Message) {
  let Model(count:) = model
  case msg {
    Increment -> {
      Model(count: count + 1)
    }
    Decrement -> {
      Model(count: count - 1)
    }
  }
}

fn view(model: Model) -> element.Element(Message) {
  let Model(count:) = model
  counter(count)
}

fn counter(count: Int) {
  html.div([], [
    html.button([event.on_click(Increment)], [html.text("Increment")]),
    html.div([], [html.text("Count: " <> { count |> int.to_string })]),
    html.button([event.on_click(Decrement)], [html.text("Decrement")]),
  ])
}
