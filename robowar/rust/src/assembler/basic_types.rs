use chumsky::{Parser, error::Rich};
use chumsky::{extra::Err, prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub enum NumberLiteral {
    I64(i64),
    U64(u64),
    F64(f64),
}

pub fn number_literal<'a>() -> impl Parser<'a, &'a str, NumberLiteral, Err<Rich<'a, char>>> {
    choice((
        number_literal_f64().map(NumberLiteral::F64),
        number_literal_i64().map(NumberLiteral::I64),
        number_literal_u64().map(NumberLiteral::U64),
    ))
}

fn number_literal_i64<'a>() -> impl Parser<'a, &'a str, i64, Err<Rich<'a, char>>> {
    regex(r"-?[0-9]+").try_map(|input: &str, span| match input.parse() {
        Ok(result) => Ok(result),
        Err(e) => Err(Rich::custom(span, e)),
    })
}

fn number_literal_u64<'a>() -> impl Parser<'a, &'a str, u64, Err<Rich<'a, char>>> {
    regex(r"[0-9]+").try_map(|input: &str, span| match input.parse() {
        Ok(result) => Ok(result),
        Err(e) => Err(Rich::custom(span, e)),
    })
}

fn number_literal_f64<'a>() -> impl Parser<'a, &'a str, f64, Err<Rich<'a, char>>> {
    // TODO get a better float regex, match rust's?

    let prefix = r"[+-]?";
    let has_integer_part_optional_float_part = r"[0-9]+\.[0-9]*";
    let optional_integer_part_has_float_part = r"[0-9]*\.[0-9]+";
    let exponent_part = r"[eE][+-]?[0-9]+";

    choice((
        regex(
            format!("(?:{prefix})?{has_integer_part_optional_float_part}(?:{exponent_part})?")
                .as_str(),
        ),
        regex(
            format!("(?:{prefix})?{optional_integer_part_has_float_part}(?:{exponent_part})?")
                .as_str(),
        ),
        regex(format!("(?:{prefix})?[0-9]+{exponent_part}").as_str()),
    ))
    .try_map(|input: &str, span| match input.parse() {
        Ok(result) => Ok(result),
        Err(e) => Err(Rich::custom(span, e)),
    })
}

pub fn identifier<'a>() -> impl Parser<'a, &'a str, String, Err<Rich<'a, char>>> {
    regex(r"[a-zA-Z_][a-zA-Z0-9_]*").map(|s: &str| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_literals() {
        assert_eq!(
            number_literal().parse("42").into_result(),
            Ok(NumberLiteral::I64(42))
        );
        assert_eq!(
            number_literal().parse("-42").into_result(),
            Ok(NumberLiteral::I64(-42))
        );
        assert_eq!(
            number_literal().parse("1.5").into_result(),
            Ok(NumberLiteral::F64(1.5))
        );
        assert_eq!(
            number_literal().parse("0").into_result(),
            Ok(NumberLiteral::I64(0))
        );
        assert_eq!(
            number_literal().parse("0.").into_result(),
            Ok(NumberLiteral::F64(0.))
        );
        assert_eq!(
            number_literal().parse("18446744073709551615").into_result(),
            Ok(NumberLiteral::U64(u64::MAX))
        );
        assert!(number_literal().parse("abc").has_errors());
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            identifier().parse("foo").into_result(),
            Ok("foo".to_string())
        );
        assert_eq!(
            identifier().parse("_bar123").into_result(),
            Ok("_bar123".to_string())
        );
        assert_eq!(
            identifier().parse("A1B2_C3").into_result(),
            Ok("A1B2_C3".to_string())
        );
        assert!(identifier().parse("1abc").has_errors());
        assert!(identifier().parse("foo-bar").has_errors());
    }
}
