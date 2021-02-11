/// Specialized for Mahjong Soul.
use convlog::tenhou::RawLog;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawLogExt {
    #[serde(rename = "_target_actor")]
    pub target_actor: Option<u8>,

    #[serde(flatten)]
    pub raw_log: RawLog,
}
