use super::pai::Pai;
use serde::{Deserialize, Deserializer, Serialize};

/// Describes an event in mjlog format.
///
/// Note that this is a simplified version of mjlog, but it is sufficient for
/// akochan to read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    None,

    StartGame {
        kyoku_first: u8,
        aka_flag: bool,
        names: [String; 4],
    },
    StartKyoku {
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        bakaze: Pai,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        dora_marker: Pai,
        kyoku: u8, // counts from 1
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        #[serde(deserialize_with = "deserialize_tehais_from_str")]
        tehais: [[Pai; 13]; 4],
    },

    Tsumo {
        actor: u8,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        pai: Pai,
    },
    Dahai {
        actor: u8,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        pai: Pai,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        pai: Pai,
        #[serde(deserialize_with = "deserialize_pai_2_from_str")]
        consumed: [Pai; 2],
    },
    Pon {
        actor: u8,
        target: u8,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        pai: Pai,
        #[serde(deserialize_with = "deserialize_pai_2_from_str")]
        consumed: [Pai; 2],
    },
    Daiminkan {
        actor: u8,
        target: u8,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        pai: Pai,
        #[serde(deserialize_with = "deserialize_pai_3_from_str")]
        consumed: [Pai; 3],
    },
    Kakan {
        actor: u8,
        #[serde(deserialize_with = "deserialize_pai_from_str")]
        pai: Pai,
        #[serde(deserialize_with = "deserialize_pai_3_from_str")]
        consumed: [Pai; 3],
    },
    Ankan {
        actor: u8,
        #[serde(deserialize_with = "deserialize_pai_4_from_str")]
        consumed: [Pai; 4],
    },
    Dora {
        #[serde(deserialize_with = "deserialize_pai_from_str")]
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
    #[inline]
    pub fn actor(&self) -> Option<u8> {
        match *self {
            Event::Tsumo { actor, .. }
            | Event::Dahai { actor, .. }
            | Event::Chi { actor, .. }
            | Event::Pon { actor, .. }
            | Event::Daiminkan { actor, .. }
            | Event::Kakan { actor, .. }
            | Event::Ankan { actor, .. }
            | Event::Reach { actor, .. }
            | Event::ReachAccepted { actor, .. }
            | Event::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }

    #[inline]
    pub fn naki_target(&self) -> Option<u8> {
        match *self {
            Event::Chi { target, .. }
            | Event::Pon { target, .. }
            | Event::Daiminkan { target, .. } => Some(target),
            _ => None,
        }
    }
}

#[inline]
fn deserialize_pai_from_str<'de, D>(deserializer: D) -> Result<Pai, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Pai::from(&*String::deserialize(deserializer)?))
}

fn deserialize_pai_2_from_str<'de, D>(deserializer: D) -> Result<[Pai; 2], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <[String; 2]>::deserialize(deserializer)?;
    Ok([Pai::from(&*s[0]), Pai::from(&*s[1])])
}

fn deserialize_pai_3_from_str<'de, D>(deserializer: D) -> Result<[Pai; 3], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <[String; 3]>::deserialize(deserializer)?;
    Ok([Pai::from(&*s[0]), Pai::from(&*s[1]), Pai::from(&*s[2])])
}

fn deserialize_pai_4_from_str<'de, D>(deserializer: D) -> Result<[Pai; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <[String; 4]>::deserialize(deserializer)?;
    Ok([
        Pai::from(&*s[0]),
        Pai::from(&*s[1]),
        Pai::from(&*s[2]),
        Pai::from(&*s[3]),
    ])
}

fn deserialize_tehais_from_str<'de, D>(deserializer: D) -> Result<[[Pai; 13]; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <[[String; 13]; 4]>::deserialize(deserializer)?;
    Ok([
        [
            Pai::from(&*s[0][0]),
            Pai::from(&*s[0][1]),
            Pai::from(&*s[0][2]),
            Pai::from(&*s[0][3]),
            Pai::from(&*s[0][4]),
            Pai::from(&*s[0][5]),
            Pai::from(&*s[0][6]),
            Pai::from(&*s[0][7]),
            Pai::from(&*s[0][8]),
            Pai::from(&*s[0][9]),
            Pai::from(&*s[0][10]),
            Pai::from(&*s[0][11]),
            Pai::from(&*s[0][12]),
        ],
        [
            Pai::from(&*s[1][0]),
            Pai::from(&*s[1][1]),
            Pai::from(&*s[1][2]),
            Pai::from(&*s[1][3]),
            Pai::from(&*s[1][4]),
            Pai::from(&*s[1][5]),
            Pai::from(&*s[1][6]),
            Pai::from(&*s[1][7]),
            Pai::from(&*s[1][8]),
            Pai::from(&*s[1][9]),
            Pai::from(&*s[1][10]),
            Pai::from(&*s[1][11]),
            Pai::from(&*s[1][12]),
        ],
        [
            Pai::from(&*s[2][0]),
            Pai::from(&*s[2][1]),
            Pai::from(&*s[2][2]),
            Pai::from(&*s[2][3]),
            Pai::from(&*s[2][4]),
            Pai::from(&*s[2][5]),
            Pai::from(&*s[2][6]),
            Pai::from(&*s[2][7]),
            Pai::from(&*s[2][8]),
            Pai::from(&*s[2][9]),
            Pai::from(&*s[2][10]),
            Pai::from(&*s[2][11]),
            Pai::from(&*s[2][12]),
        ],
        [
            Pai::from(&*s[3][0]),
            Pai::from(&*s[3][1]),
            Pai::from(&*s[3][2]),
            Pai::from(&*s[3][3]),
            Pai::from(&*s[3][4]),
            Pai::from(&*s[3][5]),
            Pai::from(&*s[3][6]),
            Pai::from(&*s[3][7]),
            Pai::from(&*s[3][8]),
            Pai::from(&*s[3][9]),
            Pai::from(&*s[3][10]),
            Pai::from(&*s[3][11]),
            Pai::from(&*s[3][12]),
        ],
    ])
}
