import gleam/list
import gleam/string

const separator = "/"

pub type Path {
  Path(components: List(String))
}

pub fn new(s: String) -> Path {
  Path(string.split(s, on: separator))
}

pub fn join(a: Path, b: Path) -> Path {
  Path(list.concat([a.components, b.components]))
}

pub fn to_string(p: Path) -> String {
  string.join(p.components, with: separator)
}
