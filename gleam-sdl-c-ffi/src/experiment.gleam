import gleam/erlang/charlist
import gleam/int
import gleam/io
import gleam/string
import sdl_raw

pub fn main() -> Nil {
  io.println(
    "SDL version  (compiled): "
    <> sdl_version_int_to_string(sdl_raw.version_compiled()),
  )
  io.println(
    "SDL version  (linked):   "
    <> sdl_version_int_to_string(sdl_raw.version_linked()),
  )
  io.println(
    "SDL revision (compiled): "
    <> charlist.to_string(sdl_raw.get_revision_compiled()),
  )
  io.println(
    "SDL revision (linked):   "
    <> charlist.to_string(sdl_raw.get_revision_linked()),
  )

  let assert Ok(_) = sdl_init(sdl_raw.init_video())

  // TODO make a window
  echo sdl_raw.create_window_and_renderer(
    title: "Experiment" |> charlist.from_string,
    width: 1024,
    height: 768,
    flags: 0,
  )
    as "TODO create_window_and_renderer"

  // TODO event loop

  sdl_raw.quit()

  Nil
}

fn sdl_init(flags: sdl_raw.InitFlags) -> Result(Nil, String) {
  sdl_try(fn() {
    sdl_raw.init(flags)
    Nil
  })
}

fn sdl_try(f: fn() -> a) -> Result(a, String) {
  sdl_raw.clear_error()
  let result = f()
  let err = sdl_raw.get_error() |> charlist.to_string()
  case string.is_empty(err) {
    False -> Error(err)
    True -> Ok(result)
  }
}

fn sdl_version_int_to_string(version: Int) -> String {
  let major = sdl_raw.version_major(version)
  let minor = sdl_raw.version_minor(version)
  let micro = sdl_raw.version_micro(version)
  int.to_string(major)
  <> "."
  <> int.to_string(minor)
  <> "."
  <> int.to_string(micro)
}
