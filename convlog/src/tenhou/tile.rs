use crate::Tile;

use num_enum::TryFromPrimitive;
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};

#[derive(Debug, Clone, Copy, SerializeRepr, DeserializeRepr, TryFromPrimitive)]
#[repr(u8)]
pub enum TenhouTile {
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

impl From<TenhouTile> for Tile {
    fn from(pai: TenhouTile) -> Self {
        let n = match pai {
            TenhouTile::AkaMan5 => 34,
            TenhouTile::AkaPin5 => 35,
            TenhouTile::AkaSou5 => 36,
            TenhouTile::Unknown => 37,
            _ => {
                let id = pai as u8;
                let kind = id / 10 - 1;
                let num = id % 10 - 1;
                kind * 9 + num
            }
        };

        // SAFETY: `n` will not goes out of range.
        unsafe { Self::new_unchecked(n) }
    }
}

impl From<Tile> for TenhouTile {
    fn from(tile: Tile) -> Self {
        let n = tile.as_u8();
        let id = match n {
            // 1m..=9s
            0..=26 => {
                let kind = n / 9 + 1;
                let num = n % 9 + 1;
                kind * 10 + num
            }
            // E..=C
            27..=33 => 41 + (n - 27),
            // 5mr
            34 => 51,
            // 5pr
            35 => 52,
            // 5sr
            36 => 53,
            _ => 0,
        };
        id.try_into().unwrap()
    }
}
