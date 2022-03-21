use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use num_enum::TryFromPrimitive;
use once_cell::sync::Lazy;
use serde_repr::Deserialize_repr as DeserializeRepr;
use serde_repr::Serialize_repr as SerializeRepr;
use thiserror::Error;

/// Describes a pai in tenhou.net/6 format.
///
/// It de/serializes as an `u8` in tenhou.net/6 format.
#[derive(Debug, Clone, Copy, PartialEq, Hash, SerializeRepr, DeserializeRepr, TryFromPrimitive)]
#[repr(u8)]
pub enum Pai {
    Unknown = 0,

    Man1 = 11,
    Man2 = 12,
    Man3 = 13,
    Man4 = 14,
    Man5 = 15,
    Man6 = 16,
    Man7 = 17,
    Man8 = 18,
    Man9 = 19,

    Pin1 = 21,
    Pin2 = 22,
    Pin3 = 23,
    Pin4 = 24,
    Pin5 = 25,
    Pin6 = 26,
    Pin7 = 27,
    Pin8 = 28,
    Pin9 = 29,

    Sou1 = 31,
    Sou2 = 32,
    Sou3 = 33,
    Sou4 = 34,
    Sou5 = 35,
    Sou6 = 36,
    Sou7 = 37,
    Sou8 = 38,
    Sou9 = 39,

    East = 41,
    South = 42,
    West = 43,
    North = 44,
    Haku = 45,
    Hatsu = 46,
    Chun = 47,

    AkaMan5 = 51,
    AkaPin5 = 52,
    AkaSou5 = 53,
}

impl Eq for Pai {}

const MJAI_PAI_STRINGS: &[&str] = &[
    "?", "?", "?", "?", "?", "?", "?", "?", "?", "?", // 0~9
    "?", "1m", "2m", "3m", "4m", "5m", "6m", "7m", "8m", "9m", // 10~19
    "?", "1p", "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", // 20~29
    "?", "1s", "2s", "3s", "4s", "5s", "6s", "7s", "8s", "9s", // 30~39
    "?", "E", "S", "W", "N", "P", "F", "C", "?", "?", // 40~49
    "?", "5mr", "5pr", "5sr", // 50~53
];

static MJAI_PAI_STRINGS_MAP: Lazy<HashMap<String, Pai>> = Lazy::new(|| {
    let mut m = HashMap::new();

    for (i, &v) in MJAI_PAI_STRINGS.iter().enumerate() {
        if let Ok(pai) = Pai::try_from(i as u8) {
            m.insert(v.to_owned(), pai);
        }
    }
    assert_eq!(m.len(), 1 + 9 * 3 + 7 + 3);

    m
});

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid pai string {0:?}")]
    InvalidPaiString(String),
}

impl FromStr for Pai {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(&pai) = MJAI_PAI_STRINGS_MAP.get(s) {
            Ok(pai)
        } else {
            Err(ParseError::InvalidPaiString(s.to_owned()))
        }
    }
}

pub fn get_pais_from_str(s: &str) -> Result<Vec<Pai>, ParseError> {
    let mut pais: Vec<Pai> = Vec::new();
    let zero_digit = '0'.to_digit(10).unwrap();
    for part in s.split_inclusive(&['m', 's', 'p', 'z'][..]) {
        let mut chars = part.chars().rev();
        let base = if let Some(last_char) = chars.next() {
            match last_char {
                'm' => 10, // man
                'p' => 20, // pin
                's' => 30, // sou
                'z' => 40, //
                _ => {
                    return Err(ParseError::InvalidPaiString(s.to_owned()));
                }
            }
        } else {
            return Err(ParseError::InvalidPaiString(s.to_owned()));
        };
        for p_char in chars.rev() {
            let off = p_char.to_digit(10).unwrap() - zero_digit;
            if off == 0 {
                // 0p, 0s, 0m
                match base {
                    10 => pais.push(Pai::AkaMan5),
                    20 => pais.push(Pai::AkaPin5),
                    30 => pais.push(Pai::AkaSou5),
                    _ => unreachable!(),
                };
                continue; // handle next p_char
            }
            if let Ok(p) = Pai::from_str(MJAI_PAI_STRINGS[(base + off) as usize]) {
                pais.push(p);
            } else {
                return Err(ParseError::InvalidPaiString(s.to_owned()));
            }
        }
    }
    Ok(pais)
}

impl fmt::Display for Pai {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            MJAI_PAI_STRINGS[self.as_usize() % MJAI_PAI_STRINGS.len()]
        )
    }
}

impl Default for Pai {
    #[inline]
    fn default() -> Self {
        Self::Unknown
    }
}

impl Pai {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn as_ord(self) -> impl Ord {
        match self {
            Self::AkaMan5 => 16,
            Self::AkaPin5 => 26,
            Self::AkaSou5 => 36,

            _ => {
                let id = self.as_u8();

                if [(16..20), (26..30), (36..40)]
                    .iter()
                    .any(|range| range.contains(&id))
                {
                    id + 1
                } else {
                    id
                }
            }
        }
    }

    pub fn as_unify_u8(self) -> u8 {
        match self {
            Self::AkaMan5 => 15,
            Self::AkaPin5 => 25,
            Self::AkaSou5 => 35,
            _ => self.as_u8(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pai() {
        println!("{:?} {:?} {:?}", Pai::AkaMan5, Pai::AkaPin5, Pai::AkaSou5);
    }

    #[test]
    fn test_get_pais_from_str() {
        let case0 = "123456789p123s55m";
        let res = get_pais_from_str(case0).unwrap();
        println!("{:?}", res);
        let case0 = "1234567z19m19s199p";
        let res = get_pais_from_str(case0).unwrap();
        println!("{:?}", res);
        let case0 = "1234567z10m10s110p";
        let res = get_pais_from_str(case0).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_a() {
        let x = Pai::try_from_primitive(11u8);
        println!("{:?}", x);
        let x = Pai::try_from(11u8);
        println!("{:?}", x);
    }
}
