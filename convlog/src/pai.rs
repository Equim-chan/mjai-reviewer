use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use lazy_static::lazy_static;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Describes a pai in tenhou.net/6 format.
///
/// It deserializes from u8, but serializes to String.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct Pai(pub u8);

impl Eq for Pai {}

static MJAI_PAI_STRINGS: &[&str] = &[
    "?", "?", "?", "?", "?", "?", "?", "?", "?", "?", // 0~9
    "?", "1m", "2m", "3m", "4m", "5m", "6m", "7m", "8m", "9m", // 10~19
    "?", "1p", "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", // 20~29
    "?", "1s", "2s", "3s", "4s", "5s", "6s", "7s", "8s", "9s", // 30~39
    "?", "E", "S", "W", "N", "P", "F", "C", "?", "?", // 40~49
    "?", "5mr", "5pr", "5sr", "?", "?", "?", "?", "?", "?", // 50~59
    "?", // 60
];

lazy_static! {
    static ref MJAI_PAI_STRINGS_MAP: HashMap<String, u8> = {
        let mut m = HashMap::new();
        for (i, &v) in MJAI_PAI_STRINGS.iter().enumerate() {
            m.insert(v.to_owned(), i as u8);
        }
        m
    };
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid pai string {0:?}")]
    InvalidPaiString(String),
}

impl FromStr for Pai {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(&i) = MJAI_PAI_STRINGS_MAP.get(s) {
            Ok(Pai(i))
        } else {
            Err(ParseError::InvalidPaiString(s.to_owned()))
        }
    }
}

impl fmt::Display for Pai {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", MJAI_PAI_STRINGS[usize::from(self.0 % 60)])
    }
}

impl Serialize for Pai {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl Pai {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[inline]
    pub fn serialize_literal<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.0)
    }

    pub fn serialize_slice_literal<S, P>(pais: P, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        P: AsRef<[Self]>,
    {
        let pais_ref = pais.as_ref();
        let mut seq = serializer.serialize_seq(Some(pais_ref.len()))?;
        for e in pais_ref {
            seq.serialize_element(&e.0)?;
        }
        seq.end()
    }

    pub fn deserialize_mjai_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;
        let pai = s.parse().map_err(Error::custom)?;
        Ok(pai)
    }
}
