use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

/// [`Pai`](Pai) describes a pai in tenhou.net/6 format.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
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

impl fmt::Display for Pai {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", MJAI_PAI_STRINGS[usize::from(self.0 % 60)])
    }
}

impl Serialize for Pai {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
