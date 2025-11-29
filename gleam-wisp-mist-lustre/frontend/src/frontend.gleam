import gleam/dynamic
import gleam/dynamic/decode
import gleam/fetch
import gleam/http
import gleam/http/request
import gleam/http/response
import gleam/int
import gleam/javascript/promise
import gleam/json
import gleam/option
import gleam/result
import gleam/string
import lustre
import lustre/effect
import lustre/element
import lustre/element/html
import lustre/element/keyed
import lustre/event
import shared

type Model {
  Model(count: option.Option(Int), error: option.Option(String))
}

type Message {
  GotCountFromServer(count: Int)
  GotErrorFromServer(String)
  Increment
  Decrement
}

pub fn main() -> Nil {
  let app = lustre.application(init, update, view)
  let assert Ok(_) = lustre.start(app, "#app", Nil)
  Nil
}

fn init(_) -> #(Model, effect.Effect(Message)) {
  #(Model(count: option.None, error: option.None), make_get_count_request())
}

fn update(model: Model, msg: Message) -> #(Model, effect.Effect(Message)) {
  case msg {
    GotCountFromServer(count:) -> {
      #(Model(..model, count: option.Some(count)), effect.none())
    }

    GotErrorFromServer(error) -> {
      #(Model(..model, error: option.Some(error)), effect.none())
    }

    Increment -> #(
      Model(..model, count: option.None),
      make_update_count_request(1),
    )

    Decrement -> #(
      Model(..model, count: option.None),
      make_update_count_request(-1),
    )
  }
}

fn view(model: Model) -> element.Element(Message) {
  let Model(count:, error:) = model

  let count = case count {
    option.None -> html.div([], [html.text("Loading...")])
    option.Some(count) -> counter(count)
  }

  let error = case error {
    option.None -> element.fragment([])
    option.Some(error) ->
      html.div(
        [
          // TODO error class
        ],
        [html.text("Error: " <> error)],
      )
  }

  element.fragment([count, error])
}

fn counter(count: Int) {
  html.div([], [
    html.button([event.on_click(Increment)], [html.text("Increment")]),
    html.div([], [html.text("Count: " <> { count |> int.to_string })]),
    html.button([event.on_click(Decrement)], [html.text("Decrement")]),
  ])
}

fn make_get_count_request() -> effect.Effect(Message) {
  make_json_request(
    "/counter",
    http.Get,
    option.None,
    shared.current_state_response_decoder(),
    fn(response) {
      let response.Response(body: shared.CurrentStateResponse(count:), ..) =
        response
      GotCountFromServer(count:)
    },
    fn(_response, e) { GotErrorFromServer(string.inspect(e)) },
  )
}

fn make_update_count_request(increment_by: Int) -> effect.Effect(Message) {
  make_json_request(
    "/counter",
    http.Put,
    option.Some(
      shared.UpdateStateRequest(increment_by:)
      |> shared.update_state_request_to_json,
    ),
    shared.current_state_response_decoder(),
    fn(response) {
      echo response as "TODO update response"
      todo
    },
    fn(_response, e) { GotErrorFromServer(string.inspect(e)) },
  )
}

// current published version of rsvp didn't handle all http methods, but used their source as reference
// https://github.com/hayleigh-dot-dev/rsvp/blob/main/src/rsvp.gleam
fn make_json_request(
  path: String,
  method: http.Method,
  request_body: option.Option(json.Json),
  decoder: decode.Decoder(response_type),
  response_handler: fn(response.Response(response_type)) -> msg,
  error_response_handler: fn(
    option.Option(response.Response(dynamic.Dynamic)),
    String,
  ) ->
    msg,
) -> effect.Effect(msg) {
  // TODO config file?
  let host = "http://localhost:8001"
  case request.to(host <> path) {
    // failed to parse url, run the failure handler to turn it into a message
    Error(e) -> {
      use dispatch <- effect.from
      dispatch(error_response_handler(option.None, string.inspect(e)))
    }

    // we have a request we can make
    Ok(request) -> {
      let request = request |> request.set_method(method)

      let request =
        request
        |> request.set_header("Accept", "application/json; charset=utf-8")

      // did we have a request body?
      let request = case request_body {
        option.None -> request
        option.Some(request_body) ->
          request
          |> request.set_header(
            "Content-Type",
            "application/json; charset=utf-8",
          )
          |> request.set_body(json.to_string(request_body))
      }

      // actually send the request
      use dispatch <- effect.from
      fetch.send(request)
      // wait for response
      |> promise.try_await(fetch.read_json_body)
      // parse response
      |> promise.map(fn(response) {
        case response {
          // we failed to get a json payload, run the failure handler to turn it into a message
          Error(e) -> error_response_handler(option.None, string.inspect(e))

          // we got a json payload
          Ok(response.Response(status:, headers:, body: response_body)) -> {
            case status >= 200 && status < 300 {
              // and it's an OK status code, so try to decode it
              True -> {
                case response_body |> decode.run(decoder) {
                  // failed to decode, run the failure handler to turn it into a message
                  Error(e) ->
                    error_response_handler(
                      option.Some(response.Response(
                        status:,
                        headers:,
                        body: response_body,
                      )),
                      string.inspect(e),
                    )

                  // decoded successfully, so we can run the handler callback to turn it into some message
                  Ok(response_body) ->
                    response_handler(response.Response(
                      status:,
                      headers:,
                      body: response_body,
                    ))
                }
              }

              // bad status code, run the failure handler to turn it into a message
              False ->
                error_response_handler(
                  option.Some(response.Response(
                    status:,
                    headers:,
                    body: response_body,
                  )),
                  "Non-OK status code",
                )
            }
          }
        }
      })
      // ready to send out message back into the update loop
      |> promise.tap(dispatch)

      Nil
    }
  }
}
