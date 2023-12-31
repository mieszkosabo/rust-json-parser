use std::fmt::Display;
use std::os::unix::ffi::OsStringExt;
use std::process::exit;
use std::str::FromStr;

use std::ffi::OsString;
use std::{env, fs};

fn main() {
    let input_path = env::args()
        .nth(1)
        .expect("Usage: parser <path_to_json_file>");

    let contents = fs::read(input_path).expect("Something went wrong reading the file");
    let contents = OsString::from_vec(contents);
    let contents = match contents.to_str() {
        Some(s) => s,
        None => {
            print!("1");
            exit(1)
        }
    };
    // TODO: check if number of number of open brackets won't cause overflow
    match JSONValue::from_str(contents) {
        Ok(_) => {
            print!("0")
        }
        Err(_) => print!("1"),
    }
}

use winnow::ascii::digit0;
use winnow::ascii::{digit1, escaped};
use winnow::combinator::{alt, fail};
use winnow::combinator::{opt, separated};
use winnow::error::ErrorKind;
use winnow::error::ParserError;
use winnow::error::{ContextError, ErrMode};
use winnow::stream::AsChar;
use winnow::token::{none_of, one_of, take_while};
use winnow::PResult;
use winnow::Parser;

#[derive(Debug, PartialEq)]
enum JSONValue {
    Null,
    True,
    False,
    Number(f64),
    String(OsString),
    Array(Vec<JSONValue>),
    Object(Vec<(OsString, JSONValue)>),
}

impl FromStr for JSONValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        json_value_parser.parse(s).map_err(|e| e.to_string())
    }
}

const SPACE: u8 = 0x0020_u8;
const LINE_FEED: u8 = 0x000A_u8;
const CARRIAGE_RETURN: u8 = 0x_000D_u8;
const TAB: u8 = 0x_0009_u8;

fn json_value_parser(input: &mut &str) -> PResult<JSONValue> {
    whitespace_parser(input)?;
    let res = alt((
        "null".map(|_| JSONValue::Null),
        "true".map(|_| JSONValue::True),
        "false".map(|_| JSONValue::False),
        number_parser,
        string_parser,
        array_parser,
        object_parser,
    ))
    .parse_next(input);
    whitespace_parser(input)?;

    res
}

fn array_parser(input: &mut &str) -> PResult<JSONValue> {
    '['.parse_next(input)?;
    whitespace_parser(input)?; // this is so that "[ ]" works
    let res = separated(0.., json_value_parser, ",")
        .parse_next(input)
        .map(JSONValue::Array);
    ']'.parse_next(input)?;

    res
}

fn object_parser(input: &mut &str) -> PResult<JSONValue> {
    '{'.parse_next(input)?;
    whitespace_parser(input)?;
    let res = separated(0.., obj_key_value_parser, ",")
        .parse_next(input)
        .map(JSONValue::Object);
    '}'.parse_next(input)?;

    res
}

fn obj_key_value_parser(input: &mut &str) -> PResult<(OsString, JSONValue)> {
    whitespace_parser(input)?;
    let key = match string_parser(input) {
        Ok(JSONValue::String(s)) => s,
        Ok(_) => return Err(ErrMode::from_error_kind(input, ErrorKind::Assert)),
        Err(e) => return Err(e),
    };
    whitespace_parser(input)?;
    ':'.parse_next(input)?;
    let value = json_value_parser(input)?;

    Ok((key, value))
}

// TODO: this could be refactored
fn number_parser(input: &mut &str) -> PResult<JSONValue> {
    let is_negative = opt::<&str, char, ErrMode<ContextError>, char>('-')
        .parse_next(input)
        .map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Assert))?
        .is_some();

    alt((
        (
            one_of(['1', '2', '3', '4', '5', '6', '7', '8', '9']),
            digit0,
            opt(fraction_parser),
            opt(exponent_parser),
        )
            .map(|v| {
                format!(
                    "{}{}{}{}",
                    v.0,
                    v.1,
                    v.2.map(|f| f.to_string()).unwrap_or_default(),
                    v.3.map(|f| f.to_string()).unwrap_or_default()
                )
            }),
        ('0', opt(fraction_parser), opt(exponent_parser))
            .map(|v| format!("{}{}", v.0, v.1.map(|f| f.to_string()).unwrap_or_default())),
    ))
    .parse_next(input)
    .map(|v| {
        f64::from_str(v.as_str())
            .map(|v| JSONValue::Number(if is_negative { -v } else { v }))
            .map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Assert))
    })?
}

struct Exponent {
    is_positive: bool,
    value: u64,
}

impl Display for Exponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "e{}{}",
            if self.is_positive { '+' } else { '-' },
            self.value
        )
    }
}

fn exponent_parser(input: &mut &str) -> PResult<Exponent> {
    (one_of(['e', 'E']), opt(one_of(['+', '-'])), digit1)
        .parse_next(input)
        .map(|v| {
            let (_e, sign, digits) = v;
            let is_positive = sign.is_none() || sign == Some('+');
            let value = u64::from_str(digits).unwrap();
            Exponent { is_positive, value }
        })
}

struct Fraction(u64);
impl Display for Fraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".{:?}", self.0)
    }
}

fn fraction_parser(input: &mut &str) -> PResult<Fraction> {
    ('.', digit1)
        .parse_next(input)
        .map(|v| Fraction(u64::from_str(v.1).unwrap()))
}

fn string_parser(input: &mut &str) -> PResult<JSONValue> {
    '"'.parse_next(input)?;
    let res = escaped(
        none_of(['"', '\\', '\n', '\r', '\t', ' ']),
        '\\',
        alt((escaped_characters_parser, unicode_parser, fail)),
    )
    .parse_next(input)
    .map(|s| JSONValue::String(OsString::from(s)));
    '"'.parse_next(input)?;

    res
}

fn escaped_characters_parser<'a>(input: &mut &'a str) -> PResult<&'a str> {
    alt(("\"", "\\", "/", "b", "f", "n", "r", "t", fail)).parse_next(input)
}

fn unicode_parser<'a>(input: &mut &'a str) -> PResult<&'a str> {
    "u".parse_next(input)?;
    take_while(4, AsChar::is_hex_digit).parse_next(input)
}

fn whitespace_parser(input: &mut &str) -> PResult<()> {
    take_while(0.., (SPACE, LINE_FEED, CARRIAGE_RETURN, TAB))
        .parse_next(input)
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn works_with_whitespace_around() {
        assert_eq!(JSONValue::from_str("           null"), Ok(JSONValue::Null));
        assert_eq!(JSONValue::from_str(" true         "), Ok(JSONValue::True));
        assert_eq!(JSONValue::from_str("     false   "), Ok(JSONValue::False));
    }
    #[test]
    fn parse_lonely_literals() {
        assert_eq!(JSONValue::from_str("null"), Ok(JSONValue::Null));
        assert_eq!(JSONValue::from_str("true"), Ok(JSONValue::True));
        assert_eq!(JSONValue::from_str("false"), Ok(JSONValue::False));
        assert_eq!(JSONValue::from_str("1"), Ok(JSONValue::Number(1.0)));
        assert_eq!(JSONValue::from_str("1.0"), Ok(JSONValue::Number(1.0)));
        assert_eq!(
            JSONValue::from_str("\"hello\""),
            Ok(JSONValue::String(OsString::from("hello")))
        );
    }

    #[test]
    fn with_non_utf8() {
        assert_eq!(
            JSONValue::from_str("\"€𝄞\""),
            Ok(JSONValue::String(OsString::from("€𝄞")))
        )
    }

    #[test]
    fn ts() {
        let c = "€𝄞";

        println!("{:?}", c);
    }

    #[test]
    fn number() {
        assert_eq!(JSONValue::from_str("123"), Ok(JSONValue::Number(123.0)));
        assert_eq!(JSONValue::from_str("0"), Ok(JSONValue::Number(0.0)));
    }
}
