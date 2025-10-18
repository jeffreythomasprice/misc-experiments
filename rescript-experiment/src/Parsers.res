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

/*
 TODO list
 TODO tuple2
 TODO tuple3
 TODO anyOf
 TODO atLeast
 TODO option
 */
