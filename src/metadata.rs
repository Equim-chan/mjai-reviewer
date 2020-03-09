use chrono::{DateTime, Local};
use serde::{Serialize, Serializer};
use std::time::Duration;

#[derive(Serialize)]
pub struct Metadata<'a> {
    pub jun_pt: &'a [i32; 4],

    #[serde(serialize_with = "serialize_datetime")]
    pub now: DateTime<Local>,
    #[serde(with = "humantime_serde")]
    pub parse_time: Duration,
    #[serde(with = "humantime_serde")]
    pub convert_time: Duration,
    #[serde(with = "humantime_serde")]
    pub review_time: Duration,

    pub version: &'a str,
}

#[inline]
fn serialize_datetime<S>(datetime: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}
