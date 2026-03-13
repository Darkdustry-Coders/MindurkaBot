use std::str::FromStr;

use nom::Parser;
use nom::character::complete::{char, not_line_ending};
use nom::{
    IResult,
    character::complete::{digit1, line_ending, space0},
    sequence::delimited,
};
use rust_decimal::Decimal;

#[derive(Default, Debug, Clone)]
pub struct ParsedReport {
    pub id: Option<String>,
    pub rule: Option<Decimal>,
    pub reason: Option<String>,
}

pub fn multi_parser(input: &str) -> ParsedReport {
    return if let Some(output) = structured_parser(input) {
        output
    } else {
        ParsedReport::default()
    };
}

fn structured_parser(input: &str) -> Option<ParsedReport> {
    (
        line_parser::<String>,
        line_parser::<Decimal>,
        line_parser::<String>,
    )
        .parse(input)
        .ok()
        .map(|(_, (id, rule, reason))| ParsedReport { id, rule, reason })
}

fn line_parser<T: FromStr>(input: &str) -> IResult<&str, Option<T>> {
    let prefix = (digit1, char(')'), space0);

    delimited(prefix, not_line_ending, line_ending)
        .map(|it: &str| it.parse::<T>().ok())
        .parse(input)
}
