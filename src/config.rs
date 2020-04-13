use launchpad::RGBColor;

use serde::Deserialize;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(with = "hex_color_format")]
    pub colors: HashMap<String, RGBColor>,
}

impl Config {}

mod hex_color_format {
    use launchpad::RGBColor;

    use serde::{Deserialize, Deserializer};

    use std::collections::HashMap;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, RGBColor>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        map.into_iter()
            .map(|(k, v)| Ok((k, parse_hex_color(&v)?)))
            .collect::<Result<HashMap<String, RGBColor>, ParseHexError>>()
            .map_err(serde::de::Error::custom)
    }

    fn parse_hex_color(s: &str) -> Result<RGBColor, ParseHexError> {
        if s.len() != 6 {
            return Err(ParseHexError::InvalidLength);
        }

        Ok(RGBColor(parse_hex(&s[..2])?, parse_hex(&s[2..4])?, parse_hex(&s[4..])?))
    }

    #[derive(Debug)]
    pub enum ParseHexError {
        InvalidCharacter {
            character: char,
            index: usize,
        },
        InvalidLength,
    }

    impl std::error::Error for ParseHexError {}

    impl std::fmt::Display for ParseHexError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseHexError::InvalidCharacter {
                    character: c,
                    index: i,
                } => write!(f, "invalid character at position {}", i),
                ParseHexError::InvalidLength => write!(f, "invalid length, must be six characters"),
            }
        }
    }

    pub(super) fn parse_hex(s: &str) -> Result<u8, ParseHexError> {
        if s.len() > 2 {
            return Err(ParseHexError::InvalidLength);
        }

        let mut value = 0;

        for (i, character) in s.chars().enumerate() {
            let c = character as u8;
            let v = match c {
                b'0'..=b'9' => Ok(c - b'0'),
                b'A'..=b'F' => Ok(c - b'A' + 10),
                b'a'..=b'a' => Ok(c - b'a' + 10),
                _ => Err(ParseHexError::InvalidCharacter {
                    character,
                    index: i,
                }),
            }?;

            value &= v << (4 * i);
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_parse() {
        parse_hex("");
    }
}
