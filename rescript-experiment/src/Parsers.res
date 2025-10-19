type matchResult<'t> = {
  result: 't,
  remainder: string,
}

type matcher<'t> = string => option<matchResult<'t>>

let string = (s: string): matcher<string> => input =>
  if input->String.startsWith(s) {
    Some({
      result: input->String.substring(~start=0, ~end=s->String.length),
      remainder: input->String.substring(~start=s->String.length, ~end=input->String.length),
    })
  } else {
    None
  }

// TODO char range

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

/*
 TODO atLeast
 TODO skipPrefix
 TODO skipSuffix
 TODO skipPrefixAndSuffix
 */
