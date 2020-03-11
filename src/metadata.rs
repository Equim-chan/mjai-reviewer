use std::time::Duration;

use serde::Serialize;

#[derive(Serialize)]
pub struct Metadata<'a> {
    pub jun_pt: &'a [i32; 4],
    pub tenhou_id: Option<&'a str>,

    #[serde(with = "humantime_serde")]
    pub parse_time: Duration,
    #[serde(with = "humantime_serde")]
    pub convert_time: Duration,
    #[serde(with = "humantime_serde")]
    pub review_time: Duration,

    pub version: &'a str,
}
