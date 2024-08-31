use std::str::{CharIndices, Chars};

#[derive(Debug, Clone)]
struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn advance(&self, c: &char) -> Position {
        match c {
            '\n' => Position {
                line: self.line + 1,
                column: 0,
            },
            _ => Position {
                line: self.line,
                column: self.column + 1,
            },
        }
    }
}

struct PosStrChars<'a> {
    pos: Position,
    iterator: Chars<'a>,
}

impl<'a> Iterator for PosStrChars<'a> {
    type Item = (Position, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|c| {
            let result = (self.pos.clone(), c);
            self.pos = self.pos.advance(&c);
            result
        })
    }
}

struct PosStrCharIndices<'a> {
    pos: Position,
    iterator: CharIndices<'a>,
}

impl<'a> Iterator for PosStrCharIndices<'a> {
    type Item = (Position, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|(i, c)| {
            let result = (self.pos.clone(), i, c);
            self.pos = self.pos.advance(&c);
            result
        })
    }
}

#[derive(Debug, Clone)]
struct PosStr<'a> {
    pub pos: Position,
    pub s: &'a str,
}

impl<'a> PosStr<'a> {
    pub fn new_str(s: &'a str) -> Self {
        Self {
            pos: Position { line: 0, column: 0 },
            s,
        }
    }

    pub fn chars<'b>(&self) -> PosStrChars<'b>
    where
        'a: 'b,
    {
        PosStrChars {
            pos: self.pos.clone(),
            iterator: self.s.chars(),
        }
    }

    pub fn char_indices<'b>(&self) -> PosStrCharIndices<'b>
    where
        'a: 'b,
    {
        PosStrCharIndices {
            pos: self.pos.clone(),
            iterator: self.s.char_indices(),
        }
    }
}

impl<'a> From<&'a str> for PosStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::new_str(value)
    }
}

#[derive(Debug)]
struct Match<'a, T> {
    pub remainder: PosStr<'a>,
    pub value: T,
}

fn take_while_and_remainder<'a, F>(input: PosStr<'a>, f: F) -> Match<'a, Option<PosStr<'a>>>
where
    F: Fn(&Position, &char) -> bool,
{
    let mut last_good = None;
    for (pos, i, c) in input.char_indices() {
        if f(&pos, &c) {
            last_good = Some((pos.advance(&c), i));
        } else {
            break;
        }
    }
    match last_good {
        Some((pos, i)) => Match {
            remainder: PosStr {
                pos,
                s: &input.s[(i + 1)..],
            },
            value: Some(PosStr {
                pos: input.pos.clone(),
                s: &input.s[0..=i],
            }),
        },
        None => Match {
            remainder: input.clone(),
            value: None,
        },
    }
}

fn skip_while<'a, F>(input: PosStr<'a>, f: F) -> PosStr<'a>
where
    F: Fn(&Position, &char) -> bool,
{
    let result = take_while_and_remainder(input, f);
    result.remainder
}

fn match_multiple<'a, T, MatchF, SkipF>(
    input: PosStr<'a>,
    mf: MatchF,
    sf: SkipF,
) -> Match<'a, Vec<T>>
where
    MatchF: Fn(PosStr<'a>) -> Option<Match<'a, T>>,
    SkipF: Fn(PosStr<'a>) -> PosStr<'a>,
{
    let mut input = input;
    let mut results = Vec::new();

    // first
    match mf(input.clone()) {
        // we have at least one
        Some(Match { remainder, value }) => {
            results.push(value);
            input = remainder;
        }
        // no matches at all
        None => {
            // early exit with the empty results vector
            return Match {
                remainder: input,
                value: results,
            };
        }
    };

    // as long as we can keep matching the skip then the actual match function, add to th results
    loop {
        match mf(sf(input.clone())) {
            Some(Match { remainder, value }) => {
                results.push(value);
                input = remainder;
            }
            None => break,
        };
    }

    return Match {
        remainder: input,
        value: results,
    };
}

fn match_str<'a>(input: PosStr<'a>, s: &str) -> Option<Match<'a, &'a str>> {
    if input.s.starts_with(s) {
        Some(Match {
            remainder: PosStr {
                // TODO wrong pos
                pos: input.pos.clone(),
                s: &input.s[s.len()..],
            },
            value: &input.s[0..s.len()],
        })
    } else {
        None
    }
}

fn match_2<'a, T1, F1, T2, F2>(input: PosStr<'a>, f1: F1, f2: F2) -> Option<Match<'a, (T1, T2)>>
where
    F1: Fn(PosStr<'a>) -> Option<Match<'a, T1>>,
    F2: Fn(PosStr<'a>) -> Option<Match<'a, T2>>,
{
    let (input, result1) = match f1(input) {
        Some(Match { remainder, value }) => (remainder, value),
        None => return None,
    };
    let (input, result2) = match f2(input) {
        Some(Match { remainder, value }) => (remainder, value),
        None => return None,
    };
    Some(Match {
        remainder: input,
        value: (result1, result2),
    })
}

fn match_3<'a, T1, F1, T2, F2, T3, F3>(
    input: PosStr<'a>,
    f1: F1,
    f2: F2,
    f3: F3,
) -> Option<Match<'a, (T1, T2, T3)>>
where
    F1: Fn(PosStr<'a>) -> Option<Match<'a, T1>>,
    F2: Fn(PosStr<'a>) -> Option<Match<'a, T2>>,
    F3: Fn(PosStr<'a>) -> Option<Match<'a, T3>>,
{
    let (input, result1) = match f1(input) {
        Some(Match { remainder, value }) => (remainder, value),
        None => return None,
    };
    let (input, result2) = match f2(input) {
        Some(Match { remainder, value }) => (remainder, value),
        None => return None,
    };
    let (input, result3) = match f3(input) {
        Some(Match { remainder, value }) => (remainder, value),
        None => return None,
    };
    Some(Match {
        remainder: input,
        value: (result1, result2, result3),
    })
}

fn skip_whitespace<'a>(input: PosStr<'a>) -> PosStr<'a> {
    skip_while(input, |_pos, c| c.is_whitespace())
}

fn match_u32<'a>(input: PosStr<'a>) -> Option<Match<'a, u32>> {
    let Match { remainder, value } =
        take_while_and_remainder(input, |_pos, c| ('0'..='9').contains(c));
    value
        .map(|value| {
            value
                .s
                .parse::<u32>()
                .ok()
                .map(|value| Match { remainder, value })
        })
        .flatten()
}

fn main() {
    let input = "123, 456 ,  789 foobar";
    let results = match_multiple(input.into(), match_u32, |input| {
        match match_3(
            input.clone(),
            |input| {
                Some(Match {
                    remainder: skip_whitespace(input),
                    value: (),
                })
            },
            |input| match_str(input, ","),
            |input| {
                Some(Match {
                    remainder: skip_whitespace(input),
                    value: (),
                })
            },
        ) {
            Some(Match {
                remainder,
                value: _,
            }) => remainder,
            None => input,
        }
    });
    println!("TODO result = {:?}", results);
}
