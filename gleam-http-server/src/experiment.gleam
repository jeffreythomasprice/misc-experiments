import gleam/bit_array
import gleam/bytes_tree
import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/io
import gleam/list
import gleam/option.{None, Some}
import gleam/otp/actor
import gleam/regexp
import gleam/result
import gleam/string
import glisten

type NextConnectionIdMessage {
  Next(Subject(Int))
}

type ConnectionState {
  State(id: Int)
}

pub fn main() -> Nil {
  let assert Ok(next_connection_id) =
    actor.new(0)
    |> actor.on_message(fn(id, msg) {
      case msg {
        Next(reply) -> {
          let id = id + 1
          actor.send(reply, id)
          actor.continue(id)
        }
      }
    })
    |> actor.start()

  let assert Ok(_) =
    glisten.new(
      fn(_conn) {
        let id =
          actor.call(next_connection_id.data, 10, Next)
          |> echo as "TODO new connection id"
        #(State(id), None)
      },
      fn(state, msg, conn) {
        let _ = case msg {
          glisten.Packet(packet_bytes) -> {
            let assert Ok(_) = handle_packet(state, conn, packet_bytes)
            let assert Ok(_) =
              glisten.send(conn, bytes_tree.from_string("TODO response"))
          }
          glisten.User(_) -> {
            // TODO: Handle user messages
            todo
          }
        }
        glisten.continue(state)
      },
    )
    |> glisten.start(8000)

  process.sleep_forever()
}

fn handle_packet(state: ConnectionState, conn, packet_bytes: BitArray) {
  echo state as "TODO state"
  echo conn as "TODO conn"
  echo packet_bytes as "TODO packet"
  use packet <- result.try(
    bit_array.to_string(packet_bytes)
    |> result.map_error(fn(_) { "failed to parse request as string" }),
  )
  use request <- result.try(parse_http_request(packet))
  echo request as "TODO parsed request"
  Ok(request)
}

type Request {
  Request(StartLine, headers: Dict(String, List(String)), body: String)
}

fn parse_http_request(request: String) -> Result(Request, _) {
  let lines =
    string.split(request, on: "\n")
    |> list.map(fn(line) {
      case string.ends_with(line, "\r") {
        True -> string.drop_end(line, 1)
        False -> line
      }
    })
  use #(start_line, lines) <- result.try(parse_http_start_line(lines))
  use #(headers, remainder) <- result.try(parse_headers(lines, dict.new()))
  use body <- result.try(parse_body(remainder))
  Ok(Request(start_line, headers, body))
}

type StartLine {
  StartLine(method: String, path: String, protocol: String)
}

fn parse_http_start_line(
  lines: List(String),
) -> Result(#(StartLine, List(String)), _) {
  case lines {
    [start_line, ..remainder] -> {
      let assert Ok(r) = regexp.from_string("^([^\\s]+)\\s+(.+)\\s([^\\s]+)$")
      case regexp.scan(r, start_line) {
        [regexp.Match(_, [Some(method), Some(path), Some(protocol)])] ->
          Ok(#(StartLine(method, path, protocol), remainder))
        _ -> Error("Failed to parse HTTP start line")
      }
    }
    _ -> Error("Missing start line")
  }
}

fn parse_headers(
  lines: List(String),
  headers: Dict(String, List(String)),
) -> Result(#(Dict(String, List(String)), List(String)), _) {
  case lines {
    [header_line, ..remainder] -> {
      let assert Ok(r) = regexp.from_string("^([^:]+?): (.*)$")
      case regexp.scan(r, header_line) {
        [regexp.Match(_, [Some(name), Some(value)])] -> {
          parse_headers(
            remainder,
            dict.upsert(headers, name, fn(existing) {
              case existing {
                None -> [value]
                Some(existing) -> [value, ..existing]
              }
            }),
          )
        }
        _ -> Ok(#(headers, lines))
      }
    }
    _ -> Ok(#(headers, lines))
  }
}

fn parse_body(lines: List(String)) -> Result(String, _) {
  case lines {
    [_, ..result] -> Ok(result |> string.join("\n"))
    _ -> Ok("")
  }
}
