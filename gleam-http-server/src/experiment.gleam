import gleam/bit_array
import gleam/bytes_tree
import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/int
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

type StartLine {
  StartLine(method: String, path: String, protocol: String)
}

type Request {
  Request(
    method: String,
    path: String,
    headers: Dict(String, List(String)),
    body: String,
  )
}

type Response {
  Response(status: Int, headers: Dict(String, List(String)), body: String)
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
            let assert Ok(response) =
              handle_packet(state, conn, packet_bytes, fn(request) {
                echo request as "TODO parsed request"
                Response(
                  status: 200,
                  headers: dict.new(),
                  body: "TODO response body",
                )
              })
            echo response as "TODO response"
            let response = response |> response_to_string
            echo response as "TODO response"
            let assert Ok(_) =
              glisten.send(conn, bytes_tree.from_string(response))
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

fn handle_packet(
  state: ConnectionState,
  conn,
  packet_bytes: BitArray,
  handler: fn(Request) -> Response,
) -> Result(Response, _) {
  use packet <- result.try(
    bit_array.to_string(packet_bytes)
    |> result.map_error(fn(_) { "failed to parse request as string" }),
  )
  use request <- result.try(parse_http_request(packet))
  Ok(handler(request))
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
  use #(StartLine(method, path, _), lines) <- result.try(parse_http_start_line(
    lines,
  ))
  use #(headers, remainder) <- result.try(parse_headers(lines, dict.new()))
  use body <- result.try(parse_body(remainder))
  Ok(Request(method, path, headers, body))
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

fn response_to_string(response: Response) -> String {
  let status_line =
    "HTTP/1.1 " <> int.to_string(response.status) <> " TODO status text"

  let content_length = response.body |> string.byte_size
  let headers =
    response.headers
    |> dict.insert("Content-Length", [int.to_string(content_length)])

  let headers =
    headers
    |> dict.to_list
    |> list.flat_map(fn(header) {
      let #(key, value) = header
      value |> list.map(fn(value) { #(key, value) })
    })
    |> list.map(fn(header) {
      let #(key, value) = header
      key <> ": " <> value
    })

  [status_line]
  |> list.append(headers)
  |> list.append(["", response.body])
  |> string.join("\n")
}
