type matchResult<'t> = {
  result: 't,
  remainder: string,
}

type matcher<'t> = string => option<matchResult<'t>>

let map = (m: matcher<'t>, f: 't => 'r): matcher<'r> => input =>
  m(input)->Option.map(({result, remainder}) => {result: f(result), remainder})

let string = (s: string): matcher<string> => input =>
  if input->String.startsWith(s) {
    Some({
      result: input->String.substring(~start=0, ~end=s->String.length),
      remainder: input->String.substringToEnd(~start=s->String.length),
    })
  } else {
    None
  }

let charRange = (min: char, max: char): matcher<char> => input => {
  let head = input->OCamlCompat.String.get(0)
  if head >= min && head <= max {
    Some({result: head, remainder: input->String.substringToEnd(~start=1)})
  } else {
    None
  }
}

let rec list = (l: list<matcher<'t>>): matcher<list<'t>> => input =>
  switch l {
  | list{head} =>
    switch head(input) {
    | Some({result, remainder}) => Some({result: list{result}, remainder})
    | None => None
    }
  | list{head, ...tail} =>
    let tail = list(tail)
    switch head(input) {
    | Some({result: head, remainder}) =>
      switch tail(remainder) {
      | Some({result: tail, remainder}) => Some({result: list{head, ...tail}, remainder})
      | None => None
      }
    | None => None
    }
  | _ => None
  }

let tuple2 = (m1: matcher<'t1>, m2: matcher<'t2>): matcher<('t1, 't2)> => input =>
  m1(input)->Option.flatMap(({result: r1, remainder}) =>
    m2(remainder)->Option.flatMap(({result: r2, remainder}) => Some({result: (r1, r2), remainder}))
  )

let tuple3 = (m1: matcher<'t1>, m2: matcher<'t2>, m3: matcher<'t3>): matcher<(
  't1,
  't2,
  't3,
)> => input =>
  m1(input)->Option.flatMap(({result: r1, remainder}) =>
    m2(remainder)->Option.flatMap(({result: r2, remainder}) =>
      m3(remainder)->Option.flatMap(
        ({result: r3, remainder}) => Some({result: (r1, r2, r3), remainder}),
      )
    )
  )

let rec anyOf = (l: list<matcher<'t>>): matcher<'t> => input =>
  switch l {
  | list{head} => head(input)
  | list{head, ...tail} =>
    switch head(input) {
    | Some(result) => Some(result)
    | None => anyOf(tail)(input)
    }
  | _ => None
  }

let option = (m: matcher<'t>): matcher<option<'t>> => input =>
  switch m(input) {
  | Some({result, remainder}) => Some({result: Some(result), remainder})
  | None => Some({result: None, remainder: input})
  }

let atLeast = (m: matcher<'t>, min: int): matcher<list<'t>> => {
  let rec takeAsManyAsPossible = (m: matcher<'t>, input: string): matchResult<list<'t>> => {
    switch m(input) {
    | Some({result: head, remainder}) => {
        let {result: tail, remainder} = takeAsManyAsPossible(m, remainder)
        {result: list{head, ...tail}, remainder}
      }
    | None => {result: list{}, remainder: input}
    }
  }
  input => {
    let {result, remainder} = takeAsManyAsPossible(m, input)
    if result->List.length >= min {
      Some({result, remainder})
    } else {
      None
    }
  }
}

let skipPrefix = (prefix: matcher<'p>, m: matcher<'t>): matcher<'t> =>
  tuple2(prefix, m)->map(((_, result)) => result)

let skipSuffix = (m: matcher<'t>, suffix: matcher<'s>): matcher<'t> =>
  tuple2(m, suffix)->map(((result, _)) => result)

let skipPrefixAndSuffix = (prefix: matcher<'p>, m: matcher<'t>, suffix: matcher<'s>): matcher<'t> =>
  tuple3(prefix, m, suffix)->map(((_, result, _)) => result)
