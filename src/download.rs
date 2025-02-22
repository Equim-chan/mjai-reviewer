use anyhow::{Result, ensure};

pub fn tenhou_log(log_id: &str) -> Result<String> {
    let url = format!("https://tenhou.net/5/mjlog2json.cgi?{log_id}");

    let res = ureq::get(&url)
        .header("Referer", "https://tenhou.net/")
        .call()?;
    let status = res.status();
    ensure!(status == 200, "get tenhou log: {status}");

    let body = res.into_body().read_to_string()?;
    Ok(body)
}
