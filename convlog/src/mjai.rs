use crate::Tile;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

/// Describes an event in mjai format.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    None,

    StartGame {
        names: [String; 4],

        // akochan specific
        kyoku_first: u8,
        aka_flag: bool,
    },
    StartKyoku {
        bakaze: Tile,
        dora_marker: Tile,
        kyoku: u8, // counts from 1
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        tehais: [[Tile; 13]; 4],
    },

    Tsumo {
        actor: u8,
        pai: Tile,
    },
    Dahai {
        actor: u8,
        pai: Tile,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Pon {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Daiminkan {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Kakan {
        actor: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Ankan {
        actor: u8,
        consumed: [Tile; 4],
    },
    Dora {
        dora_marker: Tile,
    },

    Reach {
        actor: u8,
    },
    ReachAccepted {
        actor: u8,
    },

    Hora {
        actor: u8,
        target: u8,

        deltas: Option<[i32; 4]>,
        ura_markers: Option<Vec<Tile>>,
    },
    Ryukyoku {
        deltas: Option<[i32; 4]>,
    },

    EndKyoku,
    EndGame,
}

impl Event {
    #[inline]
    #[must_use]
    pub fn actor(&self) -> Option<u8> {
        match *self {
            Self::Tsumo { actor, .. }
            | Self::Dahai { actor, .. }
            | Self::Chi { actor, .. }
            | Self::Pon { actor, .. }
            | Self::Daiminkan { actor, .. }
            | Self::Kakan { actor, .. }
            | Self::Ankan { actor, .. }
            | Self::Reach { actor, .. }
            | Self::ReachAccepted { actor, .. }
            | Self::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn naki_info(&self) -> Option<(u8, Tile)> {
        match *self {
            Self::Chi { target, pai, .. }
            | Self::Pon { target, pai, .. }
            | Self::Daiminkan { target, pai, .. } => Some((target, pai)),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn naki_to_ord(&self) -> isize {
        match *self {
            Self::Chi { .. } => 0,
            Self::Pon { .. } => 1,
            _ => -1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn optional_field_deser() {
        let a = r#"{"type":"hora","actor":0,"target":0}"#;
        serde_json::from_str::<Event>(a).unwrap();
    }
}
