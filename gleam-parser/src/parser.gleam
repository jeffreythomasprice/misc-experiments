import gleam/list.{length as gleam_list_length}
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

pub fn utf_codepoint_range(
  min: UtfCodepoint,
  max: UtfCodepoint,
) -> Matcher(UtfCodepoint) {
  let min = string.utf_codepoint_to_int(min)
  let max = string.utf_codepoint_to_int(max)
  fn(input) {
    let input = string.to_utf_codepoints(input)
    case input {
      [] -> None
      [head, ..tail] -> {
        let head_int = string.utf_codepoint_to_int(head)
        let in_range = head_int >= min && head_int <= max
        case in_range {
          False -> None
          True ->
            Some(MatchResult(
              result: head,
              remainder: string.from_utf_codepoints(tail),
            ))
        }
      }
    }
  }
}

/// min and max should both have a single utf codepoint
pub fn char_range(min: String, max: String) -> Matcher(String) {
  let min = string.to_utf_codepoints(min)
  let max = string.to_utf_codepoints(max)
  assert list.length(min) == 1
  assert list.length(max) == 1
  let assert Ok(min) = min |> list.first
  let assert Ok(max) = max |> list.first
  utf_codepoint_range(min, max)
  |> map(fn(x) { string.from_utf_codepoints([x]) })
}

pub fn list(l: List(Matcher(t))) -> Matcher(List(t)) {
  fn(input) {
    case l {
      [] -> Some(MatchResult(result: [], remainder: input))
      [head, ..tail] -> {
        let tail = list(tail)
        case head(input) {
          None -> None
          Some(MatchResult(result: head, remainder:)) -> {
            case tail(remainder) {
              None -> None
              Some(MatchResult(result: tail, remainder:)) ->
                Some(MatchResult(result: [head, ..tail], remainder:))
            }
          }
        }
      }
    }
  }
}

pub fn tuple2(m1: Matcher(t1), m2: Matcher(t2)) -> Matcher(#(t1, t2)) {
  fn(input) {
    use MatchResult(result: r1, remainder:) <- option.then(m1(input))
    use MatchResult(result: r2, remainder:) <- option.then(m2(remainder))
    Some(MatchResult(result: #(r1, r2), remainder:))
  }
}

pub fn tuple3(
  m1: Matcher(t1),
  m2: Matcher(t2),
  m3: Matcher(t3),
) -> Matcher(#(t1, t2, t3)) {
  fn(input) {
    use MatchResult(result: r1, remainder:) <- option.then(m1(input))
    use MatchResult(result: r2, remainder:) <- option.then(m2(remainder))
    use MatchResult(result: r3, remainder:) <- option.then(m3(remainder))
    Some(MatchResult(result: #(r1, r2, r3), remainder:))
  }
}

pub fn any_of(l: List(Matcher(t))) -> Matcher(t) {
  fn(input) {
    case l {
      [] -> None
      [head, ..tail] -> {
        case head(input) {
          None -> any_of(tail)(input)
          Some(result) -> Some(result)
        }
      }
    }
  }
}

pub fn option(m: Matcher(t)) -> Matcher(option.Option(t)) {
  fn(input) {
    case m(input) {
      None -> Some(MatchResult(result: None, remainder: input))
      Some(MatchResult(result:, remainder:)) ->
        Some(MatchResult(result: Some(result), remainder:))
    }
  }
}

fn take_as_many_as_possible(
  m: Matcher(t),
  input: String,
) -> MatchResult(List(t)) {
  case m(input) {
    None -> MatchResult(result: [], remainder: input)
    Some(MatchResult(result: head, remainder:)) -> {
      let MatchResult(result: tail, remainder:) =
        take_as_many_as_possible(m, remainder)
      MatchResult(result: [head, ..tail], remainder:)
    }
  }
}

pub fn at_least(m: Matcher(t), min: Int) -> Matcher(List(t)) {
  fn(input) {
    let MatchResult(result:, remainder:) = take_as_many_as_possible(m, input)
    case gleam_list_length(result) >= min {
      False -> None
      True -> Some(MatchResult(result:, remainder:))
    }
  }
}

pub fn skip_prefix(prefix: Matcher(p), m: Matcher(t)) -> Matcher(t) {
  fn(input) {
    use MatchResult(result: _, remainder:) <- option.then(prefix(input))
    use MatchResult(result:, remainder:) <- option.then(m(remainder))
    Some(MatchResult(result:, remainder:))
  }
}

pub fn skip_suffix(m: Matcher(t), suffix: Matcher(s)) -> Matcher(t) {
  fn(input) {
    use MatchResult(result:, remainder:) <- option.then(m(input))
    use MatchResult(result: _, remainder:) <- option.then(suffix(remainder))
    Some(MatchResult(result:, remainder:))
  }
}

pub fn skip_prefix_and_suffix(
  prefix: Matcher(p),
  m: Matcher(t),
  suffix: Matcher(s),
) -> Matcher(t) {
  fn(input) {
    use MatchResult(result: _, remainder:) <- option.then(prefix(input))
    use MatchResult(result:, remainder:) <- option.then(m(remainder))
    use MatchResult(result: _, remainder:) <- option.then(suffix(remainder))
    Some(MatchResult(result:, remainder:))
  }
}

pub fn map(m: Matcher(t), f: fn(t) -> r) -> Matcher(r) {
  fn(input) {
    m(input)
    |> option.map(fn(x) {
      let MatchResult(result:, remainder:) = x
      MatchResult(result: f(result), remainder:)
    })
  }
}
