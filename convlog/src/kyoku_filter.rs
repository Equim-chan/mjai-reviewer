use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct KyokuFilter {
    whitelist: [Vec<u8>; 16],
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing bakaze")]
    MissingBakaze,
    #[error("missing kyoku")]
    MissingKyoku,

    #[error(r#"invalid bakaze: {0:?} (expected one of "E", "S", "W", "N")"#)]
    InvalidBakaze(String),
    #[error("invalid kyoku: {0:?}")]
    InvalidKyoku(#[source] <u8 as FromStr>::Err),
    #[error("invalid honba: {0:?}")]
    InvalidHonba(#[source] <u8 as FromStr>::Err),

    #[error("invalid kyoku range: {0:?} (expected within [1, 4])")]
    InvalidKyokuRange(u8),
}

impl FromStr for KyokuFilter {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut whitelist: [Vec<u8>; 16] = Default::default();

        for part in s.split(',') {
            let mut chars = part.chars();

            let bakaze = chars
                .next()
                .map(|p| p.to_uppercase().to_string())
                .ok_or(ParseError::MissingBakaze)?;

            let offset = match bakaze.as_ref() {
                "E" => 0,
                "S" => 4,
                "W" => 8,
                "N" => 12,
                _ => return Err(ParseError::InvalidBakaze(bakaze)),
            };
            let kyoku_num: u8 = chars
                .next()
                .ok_or(ParseError::MissingKyoku)?
                .to_string()
                .parse()
                .map_err(ParseError::InvalidKyoku)?;

            if kyoku_num < 1 || kyoku_num > 4 {
                return Err(ParseError::InvalidKyokuRange(kyoku_num));
            }

            let kyoku = offset + kyoku_num - 1;
            let honba = if let Some('.') = chars.next() {
                chars
                    .collect::<String>()
                    .parse()
                    .map_err(ParseError::InvalidHonba)?
            } else {
                0
            };

            whitelist[kyoku as usize].push(honba);
        }

        Ok(Self { whitelist })
    }
}

impl KyokuFilter {
    #[inline]
    pub fn test(&self, kyoku: u8, honba: u8) -> bool {
        if kyoku > 16 {
            return false;
        };

        self.whitelist[kyoku as usize].iter().any(|&h| h == honba)
    }
}
