use std::time::Duration;

use serde::Serialize;

#[derive(Serialize)]
pub struct Metadata<'a> {
    pub pt: &'a [i32; 4],
    pub game_length: &'a str,
    pub log_id: Option<&'a str>,

    #[serde(with = "humantime_serde")]
    pub loading_time: Duration,
    #[serde(with = "humantime_serde")]
    pub review_time: Duration,

    pub deviation_threshold: f64,
    pub total_reviewed: usize,
    pub total_tolerated: usize,
    pub total_problems: usize,
    pub score: f64,

    pub version: &'a str,
}
