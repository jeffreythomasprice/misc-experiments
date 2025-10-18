import gleam/option.{type Option, None, Some}
import gleam/string

pub type MatchResult(t) {
  MatchResult(result: t, remainder: String)
}

pub type Matcher(t) =
  fn(String) -> Option(MatchResult(t))

pub fn string_literal(s: String) -> Matcher(String) {
  fn(input) {
    case string.starts_with(input, s) {
      False -> None
      True -> {
        Some(MatchResult(
          s,
          string.slice(
            input,
            string.length(s),
            string.length(input) - string.length(s),
          ),
        ))
      }
    }
  }
}
