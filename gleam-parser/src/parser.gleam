import gleam/list
import gleam/option
import gleam/result
import gleam/string

pub type Parser(t) =
  fn(String) -> Result(#(String, t), Nil)

pub fn map(p: Parser(t), f: fn(t) -> r) -> Parser(r) {
  fn(input) {
    use #(remainder, result) <- result.try(p(input))
    Ok(#(remainder, f(result)))
  }
}

pub fn flatten(p: Parser(Result(t, Nil))) -> Parser(t) {
  fn(input) {
    case p(input) {
      // no match
      Error(_) -> Error(Nil)
      Ok(result) -> {
        let #(remainder, result) = result
        case result {
          // matched, but the inner result, the parser result, is an error
          Error(_) -> Error(Nil)
          // matched, and the parser succeeded
          Ok(result) -> Ok(#(remainder, result))
        }
      }
    }
  }
}

pub fn string(s: String) -> Parser(String) {
  fn(input) {
    case string.starts_with(input, s) {
      False -> Error(Nil)
      True -> Ok(#(string.drop_left(input, string.length(s)), s))
    }
  }
}

pub fn utf_codepoint_range(c1: UtfCodepoint, c2: UtfCodepoint) -> Parser(String) {
  let c1 = string.utf_codepoint_to_int(c1)
  let c2 = string.utf_codepoint_to_int(c2)
  fn(input) {
    case string.pop_grapheme(input) {
      // we have some character in the input
      Ok(#(first, remainder)) -> {
        // covert that first character into a code point too
        let assert [first_code_point] = string.to_utf_codepoints(first)
        let first_code_point = string.utf_codepoint_to_int(first_code_point)
        // compare against the range
        case first_code_point >= c1 && first_code_point <= c2 {
          True -> Ok(#(remainder, first))
          False -> Error(Nil)
        }
      }
      // the only other case should be that the input string is empty
      _ -> Error(Nil)
    }
  }
}

pub fn char_range(c1: String, c2: String) -> Result(Parser(String), Nil) {
  // get the single character out of each
  use #(c1, c2) <- result.try(case
    string.to_utf_codepoints(c1),
    string.to_utf_codepoints(c2)
  {
    // the only case we like is where there is exactly one codepoint in each string, so we have a first but no remainder
    [c1], [c2] -> Ok(#(c1, c2))
    // all other cases mean either at least one of them is empty or has two or more codepoint
    _, _ -> Error(Nil)
  })

  // defer to the codepoint version
  Ok(utf_codepoint_range(c1, c2))
}

// TODO regex

pub fn seq2(p1: Parser(t1), p2: Parser(t2)) -> Parser(#(t1, t2)) {
  fn(input) {
    let remainder = input
    use #(remainder, r1) <- result.try(p1(remainder))
    use #(remainder, r2) <- result.try(p2(remainder))
    Ok(#(remainder, #(r1, r2)))
  }
}

pub fn seq3(
  p1: Parser(t1),
  p2: Parser(t2),
  p3: Parser(t3),
) -> Parser(#(t1, t2, t3)) {
  fn(input) {
    let remainder = input
    use #(remainder, r1) <- result.try(p1(remainder))
    use #(remainder, r2) <- result.try(p2(remainder))
    use #(remainder, r3) <- result.try(p3(remainder))
    Ok(#(remainder, #(r1, r2, r3)))
  }
}

pub fn seq4(
  p1: Parser(t1),
  p2: Parser(t2),
  p3: Parser(t3),
  p4: Parser(t4),
) -> Parser(#(t1, t2, t3, t4)) {
  fn(input) {
    let remainder = input
    use #(remainder, r1) <- result.try(p1(remainder))
    use #(remainder, r2) <- result.try(p2(remainder))
    use #(remainder, r3) <- result.try(p3(remainder))
    use #(remainder, r4) <- result.try(p4(remainder))
    Ok(#(remainder, #(r1, r2, r3, r4)))
  }
}

pub fn any2(p1: Parser(t), p2: Parser(t)) -> Parser(t) {
  fn(input) {
    case p1(input), p2(input) {
      Ok(result), _ -> Ok(result)
      _, Ok(result) -> Ok(result)
      _, _ -> Error(Nil)
    }
  }
}

pub fn any3(p1: Parser(t), p2: Parser(t), p3: Parser(t)) -> Parser(t) {
  fn(input) {
    case p1(input), p2(input), p3(input) {
      Ok(result), _, _ -> Ok(result)
      _, Ok(result), _ -> Ok(result)
      _, _, Ok(result) -> Ok(result)
      _, _, _ -> Error(Nil)
    }
  }
}

pub fn any4(
  p1: Parser(t),
  p2: Parser(t),
  p3: Parser(t),
  p4: Parser(t),
) -> Parser(t) {
  fn(input) {
    case p1(input), p2(input), p3(input), p4(input) {
      Ok(result), _, _, _ -> Ok(result)
      _, Ok(result), _, _ -> Ok(result)
      _, _, Ok(result), _ -> Ok(result)
      _, _, _, Ok(result) -> Ok(result)
      _, _, _, _ -> Error(Nil)
    }
  }
}

pub type RepeatOptions {
  Bounded(Int)
  Unbounded
}

pub fn repeat(
  p: Parser(t),
  min min: RepeatOptions,
  max max: RepeatOptions,
) -> Parser(List(t)) {
  let in_range = fn(count: Int) -> Bool {
    case min, max {
      Bounded(min), Bounded(max) -> count >= min && count <= max
      Bounded(min), Unbounded -> count >= min
      Unbounded, Bounded(max) -> count <= max
      Unbounded, Unbounded -> True
    }
  }

  fn(input) {
    repeat_impl(p, input, [], in_range)
    |> result.map(fn(result) {
      let #(remainder, result) = result
      #(remainder, result |> list.reverse)
    })
  }
}

fn repeat_impl(
  p: Parser(t),
  input: String,
  previous_results: List(t),
  in_range: fn(Int) -> Bool,
) -> Result(#(String, List(t)), Nil) {
  case
    p(input),
    in_range(list.length(previous_results)),
    in_range(list.length(previous_results) + 1)
  {
    // we already have a good list, but adding one more would break it, so we're done
    _, True, False -> Ok(#(input, previous_results))
    // we already have a good list, and didn't get a new match
    Error(_), True, _ -> Ok(#(input, previous_results))
    // we got another value to add, so add it to our list and recurse
    Ok(#(remainder, next_result)), _, _ ->
      repeat_impl(p, remainder, [next_result, ..previous_results], in_range)
    // nothing good is happening
    Error(_), False, _ -> Error(Nil)
  }
}

pub fn at_least(p: Parser(t), min: Int) -> Parser(List(t)) {
  repeat(p, min: Bounded(min), max: Unbounded)
}

pub fn at_most(p: Parser(t), max: Int) -> Parser(List(t)) {
  repeat(p, min: Unbounded, max: Bounded(max))
}

pub fn optional(p: Parser(t)) -> Parser(option.Option(t)) {
  fn(input) {
    case p(input) {
      Error(_) -> Ok(#(input, option.None))
      Ok(#(remainder, result)) -> Ok(#(remainder, option.Some(result)))
    }
  }
}

pub fn skip_prefix(prefix: Parser(s), p: Parser(t)) -> Parser(t) {
  seq2(prefix, p)
  |> map(fn(r) {
    let #(_, result) = r
    result
  })
}

pub fn skip_suffix(p: Parser(t), suffix: Parser(s)) -> Parser(t) {
  seq2(p, suffix)
  |> map(fn(r) {
    let #(result, _) = r
    result
  })
}

pub fn skip_prefix_and_suffix(
  prefix: Parser(s1),
  p: Parser(t),
  suffix: Parser(s2),
) -> Parser(t) {
  seq3(prefix, p, suffix)
  |> map(fn(r) {
    let #(_, result, _) = r
    result
  })
}
