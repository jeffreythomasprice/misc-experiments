pub mod matchers;
pub mod strings;

#[cfg(test)]
pub mod test {
    use crate::{
        matchers::{match2, repeat, take_while, MapError, Mappable, Matcher},
        strings::{Match, PosStr, Position},
    };

    fn match_u32<'a>() -> crate::matchers::MapMatcher<
        'a,
        PosStr<'a>,
        u32,
        crate::matchers::TakeWhileMatcher<impl Fn(&Position, &char) -> bool>,
        impl Fn(PosStr<'a>) -> Result<u32, MapError>,
    > {
        take_while(|_pos, c| ('0'..='9').contains(c))
            .map(|value| value.s.parse::<u32>().map_err(|e| MapError))
    }

    fn tokenize<'a, M, T>(
        m: M,
    ) -> crate::matchers::MapMatcher<
        'a,
        (T, PosStr<'a>),
        T,
        crate::matchers::Match2<
            M,
            crate::matchers::TakeWhileMatcher<impl Fn(&Position, &char) -> bool>,
        >,
        impl Fn((T, PosStr<'a>)) -> Result<T, MapError>,
    >
    where
        M: Matcher<'a, T>,
    {
        match2(m, take_while(|_pos, c| -> bool { c.is_whitespace() })).map(|(a, b)| Ok(a))
    }

    fn match_u32_list<'a>() -> crate::matchers::RepeatMatcher<
        crate::matchers::MapMatcher<
            'a,
            (u32, PosStr<'a>),
            u32,
            crate::matchers::Match2<
                crate::matchers::MapMatcher<
                    'a,
                    PosStr<'a>,
                    u32,
                    crate::matchers::TakeWhileMatcher<impl Fn(&Position, &char) -> bool>,
                    impl Fn(PosStr<'a>) -> Result<u32, MapError>,
                >,
                crate::matchers::TakeWhileMatcher<impl Fn(&Position, &char) -> bool + 'a>,
            >,
            impl Fn((u32, PosStr<'a>)) -> Result<u32, MapError>,
        >,
        u32,
        std::ops::RangeFull,
    > {
        repeat(tokenize(match_u32()), ..)
    }

    #[test]
    pub fn u32_success() {
        assert_eq!(
            match_u32().apply("123".into()),
            Some(Match {
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: ""
                },
                value: 123
            })
        );
    }

    #[test]
    pub fn u32_failure() {
        assert_eq!(match_u32().apply("foobar".into()), None);
    }

    #[test]
    pub fn u32_failure_too_big() {
        assert_eq!(match_u32().apply("4294967296".into()), None);
    }

    #[test]
    pub fn u32_list_success() {
        assert_eq!(
            match_u32_list().apply("123 456    789   foobar".into()),
            Some(Match {
                remainder: PosStr {
                    pos: Position {
                        line: 0,
                        column: 17
                    },
                    s: "foobar"
                },
                value: vec![123, 456, 789],
            })
        );
    }
}

// TODO make into tests

// // TODO make this a Matcher, impl with a TakeWhileMatcher
// fn skip_whitespace<'a>(input: PosStr<'a>) -> PosStr<'a> {
//     skip_while(input, |_pos, c| c.is_whitespace())
// }

// // TODO make this a Matcher
// fn match_u32<'a>(input: PosStr<'a>) -> Option<Match<'a, u32>> {
//     let Match { remainder, value } =
//         take_while_and_remainder(input, |_pos, c| ('0'..='9').contains(c));
//     value
//         .map(|value| {
//             value
//                 .s
//                 .parse::<u32>()
//                 .ok()
//                 .map(|value| Match { remainder, value })
//         })
//         .flatten()
// }

// fn main() {
//     let input = "123, 456 ,  789 foobar";
//     let results = match_multiple(input.into(), match_u32, |input| {
//         match match_3(
//             input.clone(),
//             |input| {
//                 Some(Match {
//                     remainder: skip_whitespace(input),
//                     value: (),
//                 })
//             },
//             |input| match_str(input, ","),
//             |input| {
//                 Some(Match {
//                     remainder: skip_whitespace(input),
//                     value: (),
//                 })
//             },
//         ) {
//             Some(Match {
//                 remainder,
//                 value: _,
//             }) => remainder,
//             None => input,
//         }
//     });
//     println!("TODO result = {:?}", results);
// }
