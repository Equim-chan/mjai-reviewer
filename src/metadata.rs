use serde::Serialize;

#[derive(Serialize)]
pub struct Metadata<'a> {
    pub now: &'a str,
    pub parse_time_us: i64,
    pub convert_time_us: i64,
    pub review_time_ms: i64,
    pub jun_pt: &'a [i32],
}
