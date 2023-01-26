use std::time::Duration;

use anyhow::{ensure, Result};
use ureq::AgentBuilder;

pub fn tenhou_log(log_id: &str) -> Result<String> {
    let url = format!("https://tenhou.net/5/mjlog2json.cgi?{log_id}");

    let agent = AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .build();

    let res = agent
        .get(&url)
        .set("Referer", "https://tenhou.net/")
        .call()?;
    let status = res.status();
    ensure!(
        status == 200,
        "get tenhou log: {status} {}",
        res.status_text(),
    );

    let body = res.into_string()?;
    Ok(body)
}
