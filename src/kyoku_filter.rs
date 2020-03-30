use std::str::FromStr;

use anyhow::anyhow;
use anyhow::{Context, Error};

#[derive(Debug, Clone)]
pub struct KyokuFilter {
    whitelist: [Vec<u8>; 16],
}

impl FromStr for KyokuFilter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut whitelist: [Vec<u8>; 16] = Default::default();

        for part in s.split(',') {
            let mut chars = part.chars();

            let bakaze = chars
                .next()
                .map(|p| p.to_uppercase().to_string())
                .context("missing bakaze")?;

            let offset = match bakaze.as_ref() {
                "E" => 0,
                "S" => 4,
                "W" => 8,
                "N" => 12,
                _ => return Err(anyhow!("invalid bakaze {}", bakaze)),
            };
            let kyoku_num: u8 = chars.next().context("missing kyoku")?.to_string().parse()?;

            if kyoku_num < 1 || kyoku_num > 4 {
                return Err(anyhow!(
                    "invalid kyoku: must be within [1, 4], got {}",
                    kyoku_num
                ));
            }

            let kyoku = offset + kyoku_num - 1;
            let honba = if let Some('.') = chars.next() {
                chars.collect::<String>().parse()?
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
