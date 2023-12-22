use std::str::FromStr;

use std::ffi::OsString;
use std::{env, fs};

fn main() {
    let input_path = env::args()
        .nth(1)
        .expect("Usage: parser <path_to_json_file>");

    let contents = fs::read_to_string(input_path).expect("Something went wrong reading the file");

    match JSONValue::from_str(&contents) {
        Ok(_) => print!("0"),
        Err(_) => print!("1"),
    }
}

use winnow::ascii::escaped;
use winnow::ascii::float;
use winnow::combinator::alt;
use winnow::combinator::separated;
use winnow::error::ErrMode;
use winnow::error::ErrorKind;
use winnow::error::ParserError;
use winnow::token::one_of;
use winnow::token::{none_of, take_while};
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
        null_parser,
        true_parser,
        false_parser,
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

fn number_parser(input: &mut &str) -> PResult<JSONValue> {
    float.parse_next(input).map(JSONValue::Number)
}

fn string_parser(input: &mut &str) -> PResult<JSONValue> {
    '"'.parse_next(input)?;
    let res = escaped(
        none_of(['"', '\\']),
        '\\',
        one_of(['"', '\\', '/', 'b', 'f', 'n', 'r', 't']), // TODO: unicode
    )
    .parse_next(input)
    .map(|s| JSONValue::String(OsString::from(s)));
    '"'.parse_next(input)?;

    res
}

fn whitespace_parser(input: &mut &str) -> PResult<()> {
    take_while(0.., (SPACE, LINE_FEED, CARRIAGE_RETURN, TAB))
        .parse_next(input)
        .map(|_| ())
}

fn null_parser(input: &mut &str) -> PResult<JSONValue> {
    "null".parse_next(input).map(|_| JSONValue::Null)
}
fn true_parser(input: &mut &str) -> PResult<JSONValue> {
    "true".parse_next(input).map(|_| JSONValue::True)
}
fn false_parser(input: &mut &str) -> PResult<JSONValue> {
    "false".parse_next(input).map(|_| JSONValue::False)
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
}
