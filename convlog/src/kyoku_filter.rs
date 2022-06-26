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

            let offset = match bakaze.as_str() {
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

            if !(1..=4).contains(&kyoku_num) {
                return Err(ParseError::InvalidKyokuRange(kyoku_num));
            }

            let kyoku = offset + kyoku_num - 1;
            let honba = if chars.next() == Some('.') {
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
    #[must_use]
    pub fn test(&self, kyoku: u8, honba: u8) -> bool {
        kyoku < 16 && self.whitelist[kyoku as usize].contains(&honba)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn filter() {
        let kf: KyokuFilter = "E1,E4,S3.1".parse().unwrap();
        assert!(kf.test(0, 0));
        assert!(!kf.test(0, 1));
        assert!(!kf.test(6, 0));
        assert!(kf.test(6, 1));

        let kf: KyokuFilter = "e3.11".parse().unwrap();
        assert!(!kf.test(2, 10));
        assert!(kf.test(2, 11));

        "e9".parse::<KyokuFilter>().unwrap_err();
        "w0".parse::<KyokuFilter>().unwrap_err();
        "".parse::<KyokuFilter>().unwrap_err();
    }
}
