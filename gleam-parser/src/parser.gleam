import gleam/option.{type Option, None, Some}
import gleam/string

pub type MatchResult(t) {
  MatchResult(result: t, remainder: String)
}

pub type Matcher(t) =
  fn(String) -> Option(MatchResult(t))

pub fn string(s: String) -> Matcher(String) {
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

pub fn list(l: List(Matcher(t))) -> Matcher(List(t)) {
  fn(input) {
    case l {
      [] -> Some(MatchResult(result: [], remainder: input))
      [head, ..tail] -> {
        let tail = list(tail)
        case head(input) {
          None -> None
          Some(MatchResult(result: head, remainder: remainder)) -> {
            case tail(remainder) {
              None -> None
              Some(MatchResult(result: tail, remainder: remainder)) ->
                Some(MatchResult(result: [head, ..tail], remainder: remainder))
            }
          }
        }
      }
    }
  }
}

pub fn tuple2(m1: Matcher(t1), m2: Matcher(t2)) -> Matcher(#(t1, t2)) {
  fn(input) {
    use MatchResult(result: r1, remainder: remainder) <- option.then(m1(input))
    use MatchResult(result: r2, remainder: remainder) <- option.then(m2(
      remainder,
    ))
    Some(MatchResult(result: #(r1, r2), remainder: remainder))
  }
}

pub fn tuple3(
  m1: Matcher(t1),
  m2: Matcher(t2),
  m3: Matcher(t3),
) -> Matcher(#(t1, t2, t3)) {
  fn(input) {
    use MatchResult(result: r1, remainder: remainder) <- option.then(m1(input))
    use MatchResult(result: r2, remainder: remainder) <- option.then(m2(
      remainder,
    ))
    use MatchResult(result: r3, remainder: remainder) <- option.then(m3(
      remainder,
    ))
    Some(MatchResult(result: #(r1, r2, r3), remainder: remainder))
  }
}
/// TODO anyOf
/// TODO optional
/// TODO atLeast
