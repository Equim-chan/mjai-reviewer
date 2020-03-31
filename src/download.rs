use std::io::prelude::*;

use anyhow::anyhow;
use anyhow::{Context, Result};
use env_proxy;
use ureq;

pub fn download_tenhou_log(log_id: &str) -> Result<impl Read> {
    let url = format!("https://tenhou.net/5/mjlog2json.cgi?{}", log_id);
    let referer = format!("https://tenhou.net/6/?log={}", log_id);

    let mut req = ureq::get(&url);
    req.set("Referer", &referer);
    req.timeout_connect(10_000);

    if let Some(proxy_url) = env_proxy::for_url_str(&url).raw_value() {
        let proxy_str: String = proxy_url.chars().skip("http://".len()).collect();
        let proxy = ureq::Proxy::new(proxy_str).context("failed to parse proxy")?;
        req.set_proxy(proxy);
    }

    let res = req.call();
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
