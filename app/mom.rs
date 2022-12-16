use std::collections::BTreeMap;

use nom::{
    IResult,
    sequence::{delimited, terminated, pair},
    multi::many0,
    branch::alt,
    combinator::{map, map_res},
    bytes::complete::{tag, take, is_not},
    character::complete::digit1
}; // 7.1.1

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Bencode {
    Number(i64),
    ByteString(Vec<u8>),
    List(Vec<Bencode>),
    Dict(BTreeMap<Vec<u8>, Bencode>),
}

// examples:
//  "4:spam" -> spam
//  "5:hello" -> hello
//  "10:technology" -> technology
//  "2:hello" -> he
// Bencode strings are not necessarily utf-8
fn parse_string(bencode_bytes: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (remaining, num_characters) = terminated(
        map_res(digit1, |digits| String::from_utf8_lossy(digits).parse::<usize>()),
        tag(":")
    )(bencode_bytes)?;
    
    map(take(num_characters), |bytes: &[u8]| bytes.to_vec())(remaining)
}

// examples:
//  "i3e" -> 3,
//  "i-3e" -> -3
//  "i10e" -> 10
//  "i2562e" -> 2562
fn parse_number(bencode_bytes: &[u8]) -> IResult<&[u8], i64> {
    delimited(
        tag("i"),
        map_res(is_not("e"), |bytes| String::from_utf8_lossy(bytes).parse::<i64>()),
        tag("e")
    )(bencode_bytes)
}

// examples:
//  "l4:spam5:helloi3ee" -> [spam, hello, 3]
//  "l2:hei3eli4ei5eee -> [he, 3, [4, 5]]
fn parse_list(bencode_bytes: &[u8]) -> IResult<&[u8], Vec<Bencode>> {
    delimited(
        tag("l"),
        many0(parse_bencode),
        tag("e")
    )(bencode_bytes)
}

// examples:
//  "d 3:cow 3:moo 4:spam 4:eggs e" -> {cow: moo, spam: eggs}
//  "d 4:spam l 1:a 1:b e e" -> {spam: [a, b]}
//  "d 4:spam d 3:cow l 1:a 1:b e e e" -> {spam: {cow: [a, b]}}
fn parse_dictionary(bencode_bytes: &[u8]) -> IResult<&[u8], BTreeMap<Vec<u8>, Bencode>> {
    map(
        delimited(
            tag("d"),
            many0(
                // The `pair` combinator allows us
                // to combine capturing the output
                // of two parsers in succession
                // into a tuple!
                pair(parse_string, parse_bencode)
            ),
            tag("e")
        ),
        // `pair` captures into tuples, and
        // `many0` collects them into `Vec`s. We
        // can simply collect these to a BTreeMap.
        |elements| elements.into_iter().collect()
    )(bencode_bytes)
}

pub fn parse_bencode(bencode_bytes: &[u8]) -> IResult<&[u8], Bencode> {
    // The `alt` combinator takes a tuple of parsers and keeps running them in
    // succession until one of them succeeds, or until all of them fail.
    alt((
        // We use `map` in all four cases to wrap
        // the result of the child parsers to the
        // specific enum variant they correspond to.
        map(parse_number, Bencode::Number),
        map(parse_string, Bencode::ByteString),
        map(parse_list, Bencode::List),
        map(parse_dictionary, Bencode::Dict),
    ))(bencode_bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_byte_string() {
        assert_eq!(
            parse_bencode(b"5:hello"),
            Ok((b"" as &[u8], Bencode::ByteString("hello".into())))
        );
    }
}

fn main() {}