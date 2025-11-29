import cors_builder
import gleam/dynamic/decode
import gleam/erlang/process
import gleam/http
import gleam/http/request
import gleam/http/response
import gleam/int
import gleam/json
import gleam/otp/actor
import gleam/string
import mist
import shared
import wisp
import wisp/wisp_mist

type State {
  State(count: Int)
}

type Message {
  GetCount(reply_with: process.Subject(Int))
  UpdateCount(increment_by: Int, reply_with: process.Subject(Int))
}

pub fn main() -> Nil {
  wisp.configure_logger()
  wisp.set_logger_level(wisp.DebugLevel)

  // TODO secret key should be a config file
  let secret_key = wisp.random_string(64)

  let assert Ok(actor) =
    actor.new(State(count: 0))
    |> actor.on_message(fn(state, msg) {
      let State(count:) = state
      case msg {
        GetCount(reply_with:) -> {
          process.send(reply_with, count)
          actor.continue(state)
        }

        UpdateCount(increment_by:, reply_with:) -> {
          let new_count = count + increment_by
          process.send(reply_with, new_count)
          actor.continue(State(count: new_count))
        }
      }
    })
    |> actor.start
  let subject = actor.data

  let assert Ok(_) =
    wisp_mist.handler(fn(req) { handle_request(subject, req) }, secret_key)
    |> mist.new
    |> mist.port(8001)
    |> mist.start

  process.sleep_forever()
}

fn handle_request(
  subject: process.Subject(Message),
  req: wisp.Request,
) -> wisp.Response {
  use req <- middleware(req)

  case wisp.path_segments(req) {
    ["counter"] -> {
      case req.method {
        http.Get -> get_counter(subject, req)
        http.Put -> update_counter(subject, req)
        _ -> wisp.method_not_allowed([http.Get, http.Put])
      }
    }
    _ -> wisp.not_found()
  }
}

fn middleware(
  req: wisp.Request,
  handle_request: fn(wisp.Request) -> wisp.Response,
) -> wisp.Response {
  use <- wisp.log_request(req)

  use <- wisp.rescue_crashes

  use req <- cors_builder.wisp_middleware(
    req,
    cors_builder.new()
      |> cors_builder.allow_all_origins()
      |> cors_builder.allow_method(http.Get)
      |> cors_builder.allow_method(http.Post)
      |> cors_builder.allow_method(http.Put)
      |> cors_builder.allow_method(http.Delete)
      |> cors_builder.allow_method(http.Patch)
      |> cors_builder.allow_header("content-type"),
  )

  use req <- wisp.handle_head(req)

  handle_request(req)
}

fn get_counter(
  subject: process.Subject(Message),
  _req: wisp.Request,
) -> wisp.Response {
  get_current_count_response(subject)
}

fn update_counter(
  subject: process.Subject(Message),
  req: wisp.Request,
) -> wisp.Response {
  use json <- wisp.require_json(req)
  case decode.run(json, shared.update_state_request_decoder()) {
    Error(e) -> wisp.bad_request(string.inspect(e))
    Ok(req) -> {
      let shared.UpdateStateRequest(increment_by:) = req
      wisp.log_debug(
        "update_counter, increment_by: " <> int.to_string(increment_by),
      )
      let updated_count =
        process.call_forever(subject, fn(reply_by) {
          UpdateCount(increment_by, reply_by)
        })
      wisp.log_debug("update_counter, result: " <> int.to_string(updated_count))
      get_current_count_response(subject)
    }
  }
}

fn get_current_count_response(subject: process.Subject(Message)) {
  let count = process.call_forever(subject, GetCount)
  let response = shared.CurrentStateResponse(count:)
  let response_str =
    shared.current_state_response_to_json(response) |> json.to_string
  wisp.log_debug("response: " <> response_str)
  wisp.json_response(response_str, 200)
}
