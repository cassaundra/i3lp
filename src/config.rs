use launchpad::RGBColor;

use serde::Deserialize;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub side: Side,
    #[serde(deserialize_with = "hex_color_format::deserialize_color")]
    pub default_color: RGBColor,
    #[serde(deserialize_with = "hex_color_format::deserialize_map")]
    pub class_colors: HashMap<String, RGBColor>,
    #[serde(deserialize_with = "hex_color_format::deserialize_map")]
    pub title_colors: HashMap<String, RGBColor>,
}

impl Config {
    pub fn from_file(file: &impl AsRef<Path>) -> crate::Result<Self> {
        let contents = fs::read_to_string(file)?;
        let config = toml::de::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Bottom,
    Top,
    Left,
    Right,
}

impl Default for Side {
    fn default() -> Side {
        Side::Bottom
    }
}

mod hex_color_format {
    use launchpad::RGBColor;

    use serde::{Deserialize, Deserializer};

    use std::collections::HashMap;

    pub fn deserialize_map<'de, D>(deserializer: D) -> Result<HashMap<String, RGBColor>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        map.into_iter()
            .map(|(k, v)| Ok((k, parse_hex_color(&v)?)))
            .collect::<Result<HashMap<String, RGBColor>, ParseHexError>>()
            .map_err(serde::de::Error::custom)
    }

    pub fn deserialize_color<'de, D>(deserializer: D) -> Result<RGBColor, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_hex_color(&s).map_err(serde::de::Error::custom)
    }

    pub fn parse_hex_color(s: &str) -> Result<RGBColor, ParseHexError> {
        if s.len() != 6 {
            return Err(ParseHexError::InvalidLength);
        }

        Ok(RGBColor(parse_hex(&s[..2])?, parse_hex(&s[2..4])?, parse_hex(&s[4..])?))
    }

    #[derive(Debug, PartialEq)]
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
                    character: _c,
                    index: i,
                } => write!(f, "invalid character at position {}", i),
                ParseHexError::InvalidLength => write!(f, "invalid length, must be six characters"),
            }
        }
    }

    pub fn parse_hex(s: &str) -> Result<u8, ParseHexError> {
        if s.len() > 2 {
            return Err(ParseHexError::InvalidLength);
        }

        let mut value = 0;

        for (i, character) in s.chars().enumerate() {
            let c = character as u8;
            let v = match c {
                b'0'..=b'9' => Ok(c - b'0'),
                b'A'..=b'F' => Ok(c - b'A' + 10),
                b'a'..=b'f' => Ok(c - b'a' + 10),
                _ => Err(ParseHexError::InvalidCharacter {
                    character,
                    index: i,
                }),
            }?;

            let shift = 4 * (s.len() - i - 1);
            value |= v << shift;
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use launchpad::RGBColor;

    use super::hex_color_format::*;

    #[test]
    fn test_hex_parse() {
        assert_eq!(parse_hex("00"), Ok(0));
        assert_eq!(parse_hex("FF"), Ok(255));
        assert_eq!(parse_hex("2a"), Ok(42));

        assert_eq!(
            parse_hex("9x"),
            Err(ParseHexError::InvalidCharacter {
                character: 'x',
                index: 1,
            })
        );
        assert_eq!(parse_hex("FFFFFF"), Err(ParseHexError::InvalidLength));
    }

    #[test]
    fn test_color_parse() {
        assert_eq!(parse_hex_color("FF34A8"), Ok(RGBColor::new(255, 52, 168)));
    }
}
