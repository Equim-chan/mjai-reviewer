use crate::Pai;
use serde::Deserialize;
use serde_json::Value;
use serde_tuple::Deserialize_tuple as DeserializeTuple;

/// The overview structure of log in tenhou.net/6 format.
#[derive(Debug)]
pub struct Log {
    pub kyokus: Vec<Kyoku>,
    pub names: [String; 4],
}

/// Contains infomation about a kyoku.
#[derive(Debug)]
pub struct Kyoku {
    pub meta: kyoku::Meta,
    pub scoreboard: [i32; 4],
    pub dora_indicators: Vec<Pai>,
    pub ura_indicators: Vec<Pai>,
    pub action_tables: [ActionTable; 4],
    pub end_status: kyoku::EndStatus,
    pub hora_status: Vec<kyoku::HoraDetail>,
    pub score_deltas: [i32; 4],
}

pub mod kyoku {
    use super::*;

    #[derive(Debug, DeserializeTuple)]
    pub struct Meta {
        pub kyoku_num: u8,
        pub honba: u8,
        pub kyotaku: u8,
    }

    #[derive(Debug)]
    pub enum EndStatus {
        Hora,
        Ryukyoku,
    }

    #[derive(Debug)]
    pub struct HoraDetail {
        pub who: u8,
        pub target: u8,
    }
}

/// A group of "配牌", "取" and "出", describing a player's
/// gaming status and actions throughout a kyoku.
#[derive(Debug)]
pub struct ActionTable {
    pub haipai: [Pai; 13],
    pub takes: Vec<ActionItem>,
    pub discards: Vec<ActionItem>,
}

/// An item corresponding to each elements in "配牌", "取" and "出".
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ActionItem {
    Pai(Pai),
    Naki(String),
}

mod json_scheme {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum ResultItem {
        Status(String),
        ScoreDeltas([i32; 4]),
        HoraDetail(Value),
    }

    #[derive(Debug, DeserializeTuple)]
    pub struct Kyoku {
        pub meta: kyoku::Meta,
        pub scoreboard: [i32; 4],
        pub dora_indicators: Vec<Pai>,
        pub ura_indicators: Vec<Pai>,

        pub haipai_0: [Pai; 13],
        pub takes_0: Vec<ActionItem>,
        pub discards_0: Vec<ActionItem>,
        pub haipai_1: [Pai; 13],
        pub takes_1: Vec<ActionItem>,
        pub discards_1: Vec<ActionItem>,
        pub haipai_2: [Pai; 13],
        pub takes_2: Vec<ActionItem>,
        pub discards_2: Vec<ActionItem>,
        pub haipai_3: [Pai; 13],
        pub takes_3: Vec<ActionItem>,
        pub discards_3: Vec<ActionItem>,

        pub results: Vec<ResultItem>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Log {
        #[serde(rename = "log")]
        pub logs: Vec<Kyoku>,
        #[serde(rename = "name")]
        pub names: [String; 4],
    }
}

impl Log {
    /// Parse a tenhou.net/6 log from JSON string.
    #[inline]
    pub fn from_json_string(json_string: &str) -> serde_json::Result<Self> {
        Ok(Self::from_json_scheme(serde_json::from_str(json_string)?))
    }

    /// Parse a tenhou.net/6 log from `&[u8]`.
    #[inline]
    pub fn from_json_slice(json_slice: &[u8]) -> serde_json::Result<Self> {
        Ok(Self::from_json_scheme(serde_json::from_slice(json_slice)?))
    }

    /// Parse a tenhou.net/6 log from [`Read`].
    #[inline]
    pub fn from_json_reader<R>(json_reader: R) -> serde_json::Result<Self>
    where
        R: std::io::Read,
    {
        Ok(Self::from_json_scheme(serde_json::from_reader(
            json_reader,
        )?))
    }

    /// Parse a tenhou.net/6 log from [`serde_json::Value`].
    #[inline]
    pub fn from_json_value(json_value: serde_json::Value) -> serde_json::Result<Self> {
        Ok(Self::from_json_scheme(serde_json::from_value(json_value)?))
    }

    fn from_json_scheme(json_parsed: json_scheme::Log) -> Self {
        let json_scheme::Log { logs, names } = json_parsed;

        let kyokus = logs
            .into_iter()
            .map(|log| {
                let mut item = Kyoku {
                    meta: log.meta,
                    scoreboard: log.scoreboard,
                    dora_indicators: log.dora_indicators,
                    ura_indicators: log.ura_indicators,
                    action_tables: [
                        ActionTable {
                            haipai: log.haipai_0,
                            takes: log.takes_0,
                            discards: log.discards_0,
                        },
                        ActionTable {
                            haipai: log.haipai_1,
                            takes: log.takes_1,
                            discards: log.discards_1,
                        },
                        ActionTable {
                            haipai: log.haipai_2,
                            takes: log.takes_2,
                            discards: log.discards_2,
                        },
                        ActionTable {
                            haipai: log.haipai_3,
                            takes: log.takes_3,
                            discards: log.discards_3,
                        },
                    ],
                    end_status: kyoku::EndStatus::Ryukyoku,
                    hora_status: vec![],
                    score_deltas: [0, 0, 0, 0],
                };

                if let Some(status) = log.results.get(0) {
                    if let json_scheme::ResultItem::Status(status_text) = status {
                        if status_text == "和了" {
                            item.end_status = kyoku::EndStatus::Hora;

                            // for multile hora, sum the deltas together
                            log.results.iter().skip(1).step_by(2).for_each(|data| {
                                if let json_scheme::ResultItem::ScoreDeltas(score_deltas) = data {
                                    item.score_deltas
                                        .iter_mut()
                                        .zip(score_deltas)
                                        .for_each(|(a, &b)| *a += b);
                                }
                            });

                            // process hora (can be multiple)
                            log.results.iter().skip(2).step_by(2).for_each(|data| {
                                if let json_scheme::ResultItem::HoraDetail(hora_detail) = data {
                                    item.hora_status.push(kyoku::HoraDetail {
                                        who: hora_detail[0].as_u64().unwrap_or(0) as u8,
                                        target: hora_detail[1].as_u64().unwrap_or(0) as u8,
                                    });
                                }
                            });
                        }
                    }
                }

                item
            })
            .collect();

        Log { kyokus, names }
    }
}
