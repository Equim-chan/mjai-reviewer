use super::pai::Pai;
use serde::{Deserialize, Serialize};

/// Describes an event in mjlog format.
///
/// Note that this is a simplified version of mjlog, but it is sufficient for
/// akochan to read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    None {
        actor: u8,
    },

    StartGame {
        kyoku_first: u8, // hanchan = 0, tonpu = 4
        names: [String; 4],
    },
    StartKyoku {
        bakaze: Pai,
        dora_marker: Pai,
        kyoku: u8, // counts from 1
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        tehais: [[Pai; 13]; 4],
    },

    Tsumo {
        actor: u8,
        pai: Pai,
    },
    Dahai {
        actor: u8,
        pai: Pai,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        pai: Pai,
        consumed: [Pai; 2],
    },
    Pon {
        actor: u8,
        target: u8,
        pai: Pai,
        consumed: [Pai; 2],
    },
    Daiminkan {
        actor: u8,
        target: u8,
        pai: Pai,
        consumed: [Pai; 3],
    },
    Kakan {
        actor: u8,
        pai: Pai,
        consumed: [Pai; 3],
    },
    Ankan {
        actor: u8,
        consumed: [Pai; 4],
    },
    Dora {
        dora_marker: Pai,
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
    },
    Ryukyoku,

    EndKyoku,
    EndGame,
}

impl Event {
    pub fn get_naki_target(&self) -> Option<u8> {
        match self {
            Event::Chi { target, .. } => Some(*target),
            Event::Pon { target, .. } => Some(*target),
            Event::Daiminkan { target, .. } => Some(*target),
            _ => None,
        }
    }
}
