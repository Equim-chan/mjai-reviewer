use anyhow::{ensure, Context, Result};

pub fn tenhou_log(log_id: &str) -> Result<String> {
    let url = format!("https://tenhou.net/5/mjlog2json.cgi?{}", log_id);

    let mut req = ureq::get(&url);
    req.set("Referer", "https://tenhou.net/");
    req.timeout_connect(10_000);
    proxy_from_env(&mut req, &url)?;

    let res = req.call();
    ensure!(
        res.ok(),
        "get tenhou log: {} {}",
        res.status(),
        res.status_text(),
    );

    let body = res.into_string()?;
    Ok(body)
}

fn proxy_from_env(req: &mut ureq::Request, url: &str) -> Result<()> {
    if let Some(proxy_url) = env_proxy::for_url_str(&url).raw_value() {
        let proxy_str: String = proxy_url.chars().skip("http://".len()).collect();
        let proxy = ureq::Proxy::new(proxy_str).context("failed to parse proxy")?;
        req.set_proxy(proxy);
    }
    Ok(())
}
