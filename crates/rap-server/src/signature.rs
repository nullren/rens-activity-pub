use nom::combinator::map;
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, none_of},
    combinator::opt,
    sequence::{delimited, pair, separated_pair, terminated},
    IResult,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

fn token(input: &str) -> IResult<&str, &str> {
    nom::character::complete::multispace0(input)?;
    alphanumeric1(input)
}

fn qdtext(input: &str) -> IResult<&str, char> {
    none_of("\\\"")(input)
}

fn quoted_pair(input: &str) -> IResult<&str, char> {
    delimited(tag("\\"), nom::character::complete::anychar, multispace0)(input)
}

fn quoted_string(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        nom::multi::many0(nom::branch::alt((qdtext, quoted_pair))),
        tag("\""),
    )(input)
    .map(|(remaining, vec)| (remaining, vec.into_iter().collect()))
}

fn param(input: &str) -> IResult<&str, (&str, String)> {
    separated_pair(
        token,
        terminated(tag("="), multispace0),
        nom::branch::alt((map(token, String::from), quoted_string)),
    )(input)
}

fn comma(input: &str) -> IResult<&str, &str> {
    delimited(multispace0, tag(","), multispace0)(input)
}

fn buggy_prefix(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("Signature "))(input)
}

fn params(input: &str) -> IResult<&str, Vec<(&str, String)>> {
    pair(buggy_prefix, nom::multi::separated_list0(comma, param))(input)
        .map(|(remaining, (_prefix, list))| (remaining, list))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Signature {
    pub key_id: String,
    pub headers: Vec<String>,
    pub signature: String,
}

impl Signature {
    pub fn from_headers(signature: &str) -> Result<Self, Box<dyn Error + '_>> {
        let (_, params) = params(signature)?;
        let mut key_id = None;
        let mut headers = None;
        let mut signature = None;
        for (key, value) in params {
            match key {
                "keyId" => key_id = Some(value),
                "headers" => headers = Some(value),
                "signature" => signature = Some(value),
                _ => {}
            }
        }
        let key_id = key_id.ok_or("keyId not found")?;
        let headers = headers.ok_or("headers not found")?;
        let signature = signature.ok_or("signature not found")?;
        Ok(Self {
            key_id,
            headers: headers.split(' ').map(|s| s.to_string()).collect(),
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    fn assert_params(input: &str, expected: &[(&str, &str)]) {
        let (_, parsed) = super::params(input).unwrap();
        let parsed: Vec<(&str, &str)> = parsed
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_params() {
        assert_params(
            r#"keyId="https://example.com/users/Ren\",with,quote",headers="(request-target) host date",signature="base64encodedsignature""#,
            &[
                ("keyId", "https://example.com/users/Ren\",with,quote"),
                ("headers", "(request-target) host date"),
                ("signature", "base64encodedsignature"),
            ],
        );
    }
}
