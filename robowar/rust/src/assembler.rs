use std::sync::LazyLock;

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn advance_char(&self, c: char) -> Self {
        if c == '\n' {
            Self {
                line: self.line + 1,
                column: 0,
            }
        } else if c.is_control() && !c.is_whitespace() {
            self.clone()
        } else {
            Self {
                line: self.line,
                column: self.column + 1,
            }
        }
    }

    pub fn advance_str(&self, s: &str) -> Self {
        let mut result = self.clone();
        for c in s.chars() {
            result = result.advance_char(c);
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputString<'a> {
    input: &'a str,
    location: Location,
}

impl<'a> InputString<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            location: Location { line: 0, column: 0 },
        }
    }

    pub fn advance_len(&self, len: usize) -> Self {
        Self {
            input: &self.input[len..],
            location: self.location.advance_str(&self.input[0..len]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Error {
    pub message: String,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq)]
struct Success<'a, T> {
    pub result: T,
    pub remainder: InputString<'a>,
}

type Result<'a, T> = std::result::Result<Success<'a, T>, Error>;

pub fn assemble(input: &str) {
    todo!()
}

fn parse_literal_str<'a, 'b>(input: &InputString<'a>, s: &'b str) -> Result<'a, &'b str> {
    if input.input.starts_with(s) {
        let remainder = input.advance_len(s.len());
        Ok(Success { result: s, remainder })
    } else {
        Err(Error {
            message: format!("expected \"{}\"", s),
            location: input.location,
        })
    }
}

fn parse_regex<'a>(input: &InputString<'a>, r: &Regex) -> Result<'a, &'a str> {
    match r.find(input.input) {
        Some(m) if m.start() == 0 => {
            let result = m.as_str();
            Ok(Success {
                result,
                remainder: input.advance_len(result.len()),
            })
        }
        _ => Err(Error {
            message: format!("expected {:?}", r),
            location: input.location,
        }),
    }
}

fn skip_whitespace<'a>(input: &InputString<'a>) -> InputString<'a> {
    static R: LazyLock<Regex> = LazyLock::new(|| Regex::new("[ \t\n\r]*").unwrap());
    match parse_regex(input, &R) {
        Ok(Success { result: _, remainder }) => remainder,
        Err(_) => input.clone(),
    }
}

fn parse_identifier<'a>(input: &InputString<'a>) -> Result<'a, &'a str> {
    static R: LazyLock<Regex> = LazyLock::new(|| Regex::new("[a-zA-Z_][a-zA-Z0-9_]*").unwrap());
    parse_regex(input, &R).map_err(|_| Error {
        message: "expected identifier".to_string(),
        location: input.location,
    })
}

fn parse_label<'a>(input: &InputString<'a>) -> Result<'a, Option<&'a str>> {
    match parse_identifier(input) {
        Ok(Success {
            result: identifier,
            remainder,
        }) => match parse_literal_str(&remainder, ":") {
            Ok(Success { result: _, remainder }) => Ok(Success {
                result: Some(identifier),
                remainder,
            }),
            Err(_) => Ok(Success {
                result: None,
                remainder: input.clone(),
            }),
        },
        Err(_) => Ok(Success {
            result: None,
            remainder: input.clone(),
        }),
    }
}

fn parse_i32<'a>(input: &InputString<'a>) -> Result<'a, i32> {
    static R: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\-?[0-9_]+").unwrap());
    if let Ok(Success { result, remainder }) = parse_regex(input, &R) {
        if let Ok(result) = result.parse() {
            return Ok(Success { result, remainder });
        }
    }
    Err(Error {
        message: "expected i32".to_string(),
        location: input.location,
    })
}

// TODO more number types?
// TODO parse_number
// TODO parse_argument = identifier | number

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_advance_char_newline() {
        let result = Location { line: 2, column: 3 }.advance_char('\n');
        assert_eq!(result, Location { line: 3, column: 0 });
    }

    #[test]
    fn location_advance_char_control() {
        let result = Location { line: 2, column: 3 }.advance_char('\0');
        assert_eq!(result, Location { line: 2, column: 3 });
    }

    #[test]
    fn location_advance_char_tab() {
        let result = Location { line: 2, column: 3 }.advance_char('\t');
        assert_eq!(result, Location { line: 2, column: 4 });
    }

    #[test]
    fn location_advance_char_ascii() {
        let result = Location { line: 2, column: 3 }.advance_char('a');
        assert_eq!(result, Location { line: 2, column: 4 });
    }

    #[test]
    fn location_advance_str() {
        let result = Location { line: 2, column: 3 }.advance_str("foo\nbar baz");
        assert_eq!(result, Location { line: 3, column: 7 });
    }

    #[test]
    fn parse_literal_str_success() {
        let result = parse_literal_str(
            &InputString {
                input: "foobar",
                location: Location { line: 0, column: 0 },
            },
            "foo",
        );
        assert_eq!(
            result,
            Ok(Success {
                result: "foo",
                remainder: InputString {
                    input: "bar",
                    location: Location { line: 0, column: 3 }
                }
            })
        );
    }

    #[test]
    fn parse_literal_str_failure() {
        let result = parse_literal_str(
            &InputString {
                input: "foobar",
                location: Location { line: 0, column: 0 },
            },
            "bar",
        );
        assert_eq!(
            result,
            Err(Error {
                message: "expected \"bar\"".to_string(),
                location: Location { line: 0, column: 0 }
            })
        );
    }

    #[test]
    fn parse_regex_success() {
        let r = Regex::new("[a-z]+").unwrap();
        let result = parse_regex(
            &InputString {
                input: "foo 123",
                location: Location { line: 0, column: 0 },
            },
            &r,
        );
        assert_eq!(
            result,
            Ok(Success {
                result: "foo",
                remainder: InputString {
                    input: " 123",
                    location: Location { line: 0, column: 3 }
                }
            })
        );
    }

    #[test]
    fn parse_regex_failure() {
        let r = Regex::new("[a-z]+").unwrap();
        let result = parse_regex(
            &InputString {
                input: "123 foo",
                location: Location { line: 0, column: 0 },
            },
            &r,
        );
        assert_eq!(
            result,
            Err(Error {
                message: "expected Regex(\"[a-z]+\")".to_string(),
                location: Location { line: 0, column: 0 }
            })
        );
    }

    #[test]
    fn parse_regex_failure_not_at_start() {
        let r = Regex::new("[a-z]+").unwrap();
        let result = parse_regex(
            &InputString {
                input: " abc foo",
                location: Location { line: 0, column: 0 },
            },
            &r,
        );
        assert_eq!(
            result,
            Err(Error {
                message: "expected Regex(\"[a-z]+\")".to_string(),
                location: Location { line: 0, column: 0 }
            })
        );
    }

    #[test]
    fn skip_whitespace_skipped() {
        let result = skip_whitespace(&InputString {
            input: " \t\n  \tfoo bar ",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            InputString {
                input: "foo bar ",
                location: Location { line: 1, column: 3 }
            }
        );
    }

    #[test]
    fn skip_whitespace_nothing_to_skip() {
        let result = skip_whitespace(&InputString {
            input: "foo bar ",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            InputString {
                input: "foo bar ",
                location: Location { line: 0, column: 0 }
            }
        );
    }

    #[test]
    fn parse_identifier_success() {
        let result = parse_identifier(&InputString {
            input: "foo bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Ok(Success {
                result: "foo",
                remainder: InputString {
                    input: " bar",
                    location: Location { line: 0, column: 3 }
                }
            })
        );
    }

    #[test]
    fn parse_identifier_failure() {
        let result = parse_identifier(&InputString {
            input: "0foo bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Err(Error {
                message: "expected identifier".to_string(),
                location: Location { line: 0, column: 0 }
            })
        );
    }

    #[test]
    fn parse_label_success() {
        let result = parse_label(&InputString {
            input: "foo: bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Ok(Success {
                result: Some("foo"),
                remainder: InputString {
                    input: " bar",
                    location: Location { line: 0, column: 4 }
                }
            })
        );
    }

    #[test]
    fn parse_label_failure() {
        let result = parse_label(&InputString {
            input: "foo bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Ok(Success {
                result: None,
                remainder: InputString {
                    input: "foo bar",
                    location: Location { line: 0, column: 0 }
                }
            })
        );
    }

    #[test]
    fn parse_i32_success_1() {
        let result = parse_i32(&InputString {
            input: "123 bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Ok(Success {
                result: 123,
                remainder: InputString {
                    input: " bar",
                    location: Location { line: 0, column: 3 }
                }
            })
        );
    }

    #[test]
    fn parse_i32_success_2() {
        let result = parse_i32(&InputString {
            input: "-123 bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Ok(Success {
                result: -123,
                remainder: InputString {
                    input: " bar",
                    location: Location { line: 0, column: 4 }
                }
            })
        );
    }

    #[test]
    fn parse_i32_failure() {
        let result = parse_i32(&InputString {
            input: "foo bar",
            location: Location { line: 0, column: 0 },
        });
        assert_eq!(
            result,
            Err(Error {
                message: "expected i32".to_string(),
                location: Location { line: 0, column: 0 },
            })
        );
    }
}
