use std::io::Read;

use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(rename = "document")]
struct Document(Vec<Value>);

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Value {
    Object(Vec<Pair>),
    Number(String),
    Array(Vec<Value>),
    String(Option<StringContent>),
    Null,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename = "pair", rename_all = "snake_case")]
struct Pair {
    key: StringContainer,
    value: Box<Value>,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename = "string")]
struct StringContainer(Option<StringContent>);

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename = "string_content")]
struct StringContent(#[serde(deserialize_with = "parse_string_content")] String);

fn next_char<I: Iterator<Item = char>, E: serde::de::Error>(
    it: &mut I,
    raw: &str,
) -> Result<char, E> {
    let Some(c) = it.next() else {
        return Err(string_format_error(raw));
    };
    Ok(c)
}

fn string_format_error<E: serde::de::Error>(raw: &str) -> E {
    E::custom(format!("Invalid string value: {}", raw))
}

fn parse_string_content<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let raw = String::deserialize(deserializer)?;
    let mut out = String::new();
    let mut it = raw.chars();
    loop {
        let Some(c) = it.next() else {
            break;
        };
        if c == '\\' {
            let c2 = next_char(&mut it, &raw)?;
            let c2 = match c2 {
                'b' => '\u{0008}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'f' => '\u{000c}',
                'u' => {
                    let u0 = next_char(&mut it, &raw)?;
                    let u1 = next_char(&mut it, &raw)?;
                    let u2 = next_char(&mut it, &raw)?;
                    let u3 = next_char(&mut it, &raw)?;
                    let code_point = u32::from_str_radix(&format!("{u0}{u1}{u2}{u3}"), 16)
                        .map_err(|_| string_format_error(&raw))?;
                    char::from_u32(code_point).ok_or_else(|| string_format_error(&raw))?
                }
                x => x,
            };
            out.push(c2);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn main() {
    let json_language = tree_sitter_json::language();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(json_language).unwrap();

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();

    let tree = parser.parse(&buf, None).unwrap();

    let _ = dbg!(serde_tree_sitter::from_tree::<Document>(&tree, &buf));
}
