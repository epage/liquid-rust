// See https://github.com/Shopify/liquid-c/blob/master/ext/liquid_c/lexer.c

use nom::character::complete as character;
use nom::{branch::*, bytes::complete::*, combinator::*, sequence::*, AsChar, IResult, Parser};

pub fn nil_literal(input: &str) -> IResult<&str, ()> {
    alt((tag("nil"), tag("null"))).map(|_| ()).parse(input)
}

pub fn empty_literal(input: &str) -> IResult<&str, ()> {
    tag("empty").map(|_| ()).parse(input)
}

pub fn blank_literal(input: &str) -> IResult<&str, ()> {
    tag("blank").map(|_| ()).parse(input)
}

pub fn bool_literal(input: &str) -> IResult<&str, bool> {
    alt((tag("true").map(|_| true), tag("false").map(|_| false)))(input)
}

pub fn integer_literal(input: &str) -> IResult<&str, i64> {
    map_res(dec_int, |s| s.parse::<i64>())(input)
}

pub fn float_literal(input: &str) -> IResult<&str, f64> {
    alt((map_res(parse_float, |s| s.parse()), special_float))(input)
}

fn parse_float(input: &str) -> IResult<&str, &str> {
    recognize(tuple((dec_int, opt(frac), exp)))(input)
}

fn dec_int(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        opt(character::char('-')),
        alt((
            character::char('0'),
            map(
                tuple((
                    character::satisfy(|c| ('1'..='9').contains(&c)),
                    take_while(AsChar::is_dec_digit),
                )),
                |t| t.0,
            ),
        )),
    )))(input)
}

fn frac(input: &str) -> IResult<&str, &str> {
    recognize(tuple((character::char('.'), parse_zero_prefixable_int)))(input)
}

fn parse_zero_prefixable_int(input: &str) -> IResult<&str, &str> {
    recognize(take_while1(AsChar::is_dec_digit))(input)
}

fn exp(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        character::one_of("eE"),
        opt(character::one_of("+-")),
        parse_zero_prefixable_int,
    )))(input)
}

fn special_float(input: &str) -> IResult<&str, f64> {
    map(
        tuple((opt(character::one_of("+-")), alt((nan, inf)))),
        |(s, f)| match s {
            Some('+') | None => f,
            Some('-') => -f,
            _ => unreachable!("one_of should prevent this"),
        },
    )(input)
}

fn inf(input: &str) -> IResult<&str, f64> {
    map(tag("inf"), |_| f64::INFINITY)(input)
}

fn nan(input: &str) -> IResult<&str, f64> {
    map(tag("nan"), |_| f64::NAN)(input)
}

pub fn string_literal(input: &str) -> IResult<&str, &str> {
    alt((
        tuple((
            character::char('\''),
            take_while(|c| c != '\''),
            character::char('\''),
        )),
        tuple((
            character::char('"'),
            take_while(|c| c != '"'),
            character::char('"'),
        )),
    ))
    .map(|(_, s, _)| s)
    .parse(input)
}
