use super::TenhouTile;
use crate::{KyokuFilter, Tile};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_tuple::{Deserialize_tuple as DeserializeTuple, Serialize_tuple as SerializeTuple};
use serde_with::{serde_as, FromInto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLog {
    #[serde(rename = "log")]
    pub(super) logs: Vec<RawKyoku>,
    #[serde(rename = "name")]
    pub(super) names: [String; 4],
    pub(super) rule: Rule,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) ratingc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) lobby: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) dan: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rate: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) sx: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct RawPartialLog<'a> {
    #[serde(flatten)]
    pub(super) parent: &'a RawLog,

    #[serde(rename = "log")]
    pub(super) logs: &'a [RawKyoku],
}

/// An item corresponding to each elements in "配牌", "取" and "出".
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionItem {
    Tile(#[serde_as(as = "FromInto<TenhouTile>")] Tile),
    Tsumogiri(u8), // must be 60
    Naki(String),
}

#[serde_as]
#[derive(Debug, Clone, SerializeTuple, DeserializeTuple)]
pub(super) struct RawKyoku {
    pub(super) meta: KyokuMeta,
    pub(super) scoreboard: [i32; 4],
    #[serde_as(as = "Vec<FromInto<TenhouTile>>")]
    pub(super) dora_indicators: Vec<Tile>,
    #[serde_as(as = "Vec<FromInto<TenhouTile>>")]
    pub(super) ura_indicators: Vec<Tile>,

    #[serde_as(as = "[FromInto<TenhouTile>; 13]")]
    pub(super) haipai_0: [Tile; 13],
    pub(super) takes_0: Vec<ActionItem>,
    pub(super) discards_0: Vec<ActionItem>,

    #[serde_as(as = "[FromInto<TenhouTile>; 13]")]
    pub(super) haipai_1: [Tile; 13],
    pub(super) takes_1: Vec<ActionItem>,
    pub(super) discards_1: Vec<ActionItem>,

    #[serde_as(as = "[FromInto<TenhouTile>; 13]")]
    pub(super) haipai_2: [Tile; 13],
    pub(super) takes_2: Vec<ActionItem>,
    pub(super) discards_2: Vec<ActionItem>,

    #[serde_as(as = "[FromInto<TenhouTile>; 13]")]
    pub(super) haipai_3: [Tile; 13],
    pub(super) takes_3: Vec<ActionItem>,
    pub(super) discards_3: Vec<ActionItem>,

    pub(super) results: Vec<ResultItem>,
}

#[derive(Debug, Clone, SerializeTuple, DeserializeTuple)]
pub struct KyokuMeta {
    pub kyoku_num: u8,
    pub honba: u8,
    pub kyotaku: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum ResultItem {
    Status(String),
    ScoreDeltas([i32; 4]),
    HoraDetail(Vec<Value>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Rule {
    pub(super) disp: String,
    pub(super) aka: u8,
    pub(super) aka51: u8,
    pub(super) aka52: u8,
    pub(super) aka53: u8,
}

impl RawLog {
    #[must_use]
    pub fn get_names(&self) -> &[String; 4] {
        &self.names
    }

    #[inline]
    pub fn hide_names(&mut self) {
        self.names
            .iter_mut()
            .zip('A'..='D')
            .for_each(|(name, alias)| {
                name.clear();
                name.push(alias);
                name.push_str("さん");
            });
    }

    #[inline]
    pub fn filter_kyokus(&mut self, kyoku_filter: &KyokuFilter) {
        self.logs
            .retain(|l| kyoku_filter.test(l.meta.kyoku_num, l.meta.honba));
    }

    /// Split one raw tenhou.net/6 log into many by kyokus.
    #[must_use]
    pub fn split_by_kyoku(&self) -> Vec<RawPartialLog<'_>> {
        let mut ret = vec![];

        for kyoku in self.logs.chunks(1) {
            let kyoku_log = RawPartialLog {
                parent: self,
                logs: kyoku,
            };

            ret.push(kyoku_log);
        }

        ret
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.logs.len()
    }
}

impl From<RawPartialLog<'_>> for RawLog {
    fn from(partial_log: RawPartialLog<'_>) -> Self {
        RawLog {
            logs: partial_log.logs.to_vec(),
            ..partial_log.parent.clone()
        }
    }
}
