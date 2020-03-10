use std::io::prelude::*;

use anyhow::anyhow;
use anyhow::Result;
use ureq;

pub fn download_tenhou_log(log_id: &str) -> Result<impl Read> {
    let res = ureq::get(&format!("https://tenhou.net/5/mjlog2json.cgi?{}", log_id))
        .set("Referer", &format!("https://tenhou.net/6/?log={}", log_id))
        .timeout_connect(10_000)
        .call();

    if res.ok() {
        Ok(res.into_reader())
    } else {
        Err(anyhow!(
            "get tenhou log: {} {}",
            res.status(),
            res.status_text()
        ))
    }
}
