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

    #[test]
    fn smaller_length() {
        assert_eq!(
            parse_bencode(b"5:helloworld"),
            Ok((b"world" as &[u8], Bencode::ByteString("hello".into())))
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(
            parse_bencode(b"0:"),
            Ok((b"" as &[u8], Bencode::ByteString("".into())))
        );
    }

    #[test]
    fn empty_string_smaller_len() {
        assert_eq!(
            parse_bencode(b"0:world"),
            Ok((b"world" as &[u8], Bencode::ByteString("".into())))
        );
    }

    #[test]
    fn whitespace_in_string() {
        assert_eq!(
            parse_bencode(b"11:hello world"),
            Ok((b"" as &[u8], Bencode::ByteString("hello world".into())))
        );
    }

    #[test]
    fn long_len() {
        assert_eq!(
            parse_bencode(b"42:helloworldprogrammedtothinkandnottofeeeeel"),
            Ok((
                b"" as &[u8],
                Bencode::ByteString("helloworldprogrammedtothinkandnottofeeeeel".into())
            ))
        );
    }

    #[test]
    fn long_len_multiple_whitespace_in_string() {
        assert_eq!(
            parse_bencode(b"50:hello world programmed to think and not to feeeeel"),
            Ok((
                b"" as &[u8],
                Bencode::ByteString("hello world programmed to think and not to feeeeel".into())
            ))
        );
    }

    #[test]
    fn negative_len_string() {
        assert!(parse_bencode(b"-2:hello").is_err());
    }

    #[test]
    fn incorrect_len_string() {
        assert!(parse_bencode(b"5:worl").is_err());
    }

    #[test]
    fn invalid_len_string() {
        assert!(parse_bencode(b"5a:hello").is_err());
    }

    #[test]
    fn positive_number() {
        assert_eq!(
            parse_bencode(b"i88e"),
            Ok((b"" as &[u8], Bencode::Number(88)))
        );
    }

    #[test]
    fn zero() {
        assert_eq!(
            parse_bencode(b"i0e"),
            Ok((b"" as &[u8], Bencode::Number(0)))
        );
    }

    #[test]
    fn negative_number() {
        assert_eq!(
            parse_bencode(b"i-88e"),
            Ok((b"" as &[u8], Bencode::Number(-88)))
        );
    }

    #[test]
    fn empty_number() {
        assert!(parse_bencode(b"ie").is_err());
    }

    #[test]
    fn only_negative_sign() {
        assert!(parse_bencode(b"i-e").is_err());
    }

    #[test]
    fn basic_list() {
        let bencode_hello = "5:hello";
        let bencode_world = "5:world";

        assert_eq!(
            parse_bencode(format!("l{}{}e", bencode_hello, bencode_world).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::List(vec![
                    Bencode::ByteString("hello".into()),
                    Bencode::ByteString("world".into())
                ])
            ))
        );
    }

    #[test]
    fn heterogenous_list() {
        let bencode_string = "5:hello";
        let bencode_number = "i8e";

        assert_eq!(
            parse_bencode(format!("l{}{}e", bencode_string, bencode_number).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::List(vec![
                    Bencode::ByteString("hello".into()),
                    Bencode::Number(8)
                ])
            ))
        );
    }

    #[test]
    fn multiple_nested_list() {
        let list_one = "l5:hello5:worlde";
        let list_two = "l5:helloi8ee";

        assert_eq!(
            parse_bencode(format!("l{}{}e", list_one, list_two).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::List(vec![
                    Bencode::List(vec![
                        Bencode::ByteString("hello".into()),
                        Bencode::ByteString("world".into())
                    ]),
                    Bencode::List(vec![
                        Bencode::ByteString("hello".into()),
                        Bencode::Number(8)
                    ])
                ])
            ))
        );
    }

    #[test]
    fn empty_list() {
        assert_eq!(
            parse_bencode(b"le"),
            Ok((b"" as &[u8], Bencode::List(vec![])))
        );
    }

    #[test]
    fn incomplete_list() {
        let bencode_number = "i8e";
        assert!(parse_bencode(format!("l{0}{0}{0}", bencode_number).as_bytes()).is_err());
    }

    #[test]
    fn list_with_invalid_element() {
        let invalid_bencode_string = "-5:hello";

        assert!(parse_bencode(format!("l{}e", invalid_bencode_string).as_bytes()).is_err());
    }

    #[test]
    fn basic_dict() {
        let key_one = "3:bar";
        let val_one = "4:spam";

        let key_two = "3:foo";
        let val_two = "i88e";

        assert_eq!(
            parse_bencode(format!("d{}{}{}{}e", key_one, val_one, key_two, val_two).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::Dict(
                    vec![
                        ("bar".into(), Bencode::ByteString("spam".into())),
                        ("foo".into(), Bencode::Number(88)),
                    ]
                        .into_iter()
                        .collect()
                )
            ))
        );
    }

    #[test]
    fn empty_dict() {
        assert_eq!(
            parse_bencode(b"de"),
            Ok((b"" as &[u8], Bencode::Dict(BTreeMap::new())))
        );
    }

    #[test]
    fn incomplete_dict() {
        let key_one = "3:foo";

        assert!(parse_bencode(format!("d{}e", key_one).as_bytes()).is_err());
    }

    #[test]
    fn dict_with_invalid_key() {
        let key_one = "-3:foo";
        let val_one = "i88e";

        assert!(parse_bencode(format!("d{}{}e", key_one, val_one).as_bytes()).is_err());
    }

    #[test]
    fn dict_with_invalid_value() {
        let key_one = "3:foo";
        let val_one = "-3:bar";

        assert!(parse_bencode(format!("d{}{}e", key_one, val_one).as_bytes()).is_err());
    }

    // Integration tests
    // TODO: check if rust has clean way for test separation

    #[test]
    fn list_of_dict() {
        let key_one = "3:foo";
        let val_one = "3:bar";

        let key_two = "3:baz";
        let val_two = "3:baz";

        let dict_str = format!("d{}{}{}{}e", key_one, val_one, key_two, val_two);
        let (_, result_dict) = parse_dictionary(&dict_str.as_bytes()).unwrap();

        assert_eq!(
            parse_bencode(format!("l{0}{0}e", dict_str).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::List(vec![
                    Bencode::Dict(result_dict.clone()),
                    Bencode::Dict(result_dict)
                ])
            ))
        );
    }

    #[test]
    fn dict_of_list() {
        let bencode_hello = "5:hello";
        let bencode_world = "5:world";

        let list_str = format!("l{}{}e", bencode_hello, bencode_world);

        let key_one = "3:foo";
        let key_two = "3:bar";

        let (_, result_list) = parse_list(&list_str.as_bytes()).unwrap();

        assert_eq!(
            parse_bencode(format!("d{}{2}{}{2}e", key_one, key_two, list_str).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::Dict(
                    vec![
                        ("foo".into(), Bencode::List(result_list.clone())),
                        ("bar".into(), Bencode::List(result_list)),
                    ]
                        .into_iter()
                        .collect()
                )
            ))
        );
    }

    #[test]
    fn multiple_nested_dicts() {
        let key_one = "3:foo";
        let val_one = "3:bar";

        let key_two = "3:baz";
        let val_two = "3:baz";

        let nested_dict_str = format!("d{}{}{}{}e", key_one, val_one, key_two, val_two);
        let (_, result_nested_dict) = parse_dictionary(&nested_dict_str.as_bytes()).unwrap();

        assert_eq!(
            parse_bencode(format!("d{}{2}{}{2}e", key_one, key_two, nested_dict_str).as_bytes()),
            Ok((
                b"" as &[u8],
                Bencode::Dict(
                    vec![
                        ("foo".into(), Bencode::Dict(result_nested_dict.clone())),
                        ("baz".into(), Bencode::Dict(result_nested_dict)),
                    ]
                        .into_iter()
                        .collect()
                )
            ))
        );
    }
}