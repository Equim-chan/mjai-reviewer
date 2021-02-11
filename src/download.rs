use anyhow::anyhow;
use anyhow::{Context, Result};
use url::form_urlencoded::Serializer;

pub fn tenhou_log(log_id: &str) -> Result<String> {
    let url = format!("https://tenhou.net/5/mjlog2json.cgi?{}", log_id);

    let mut req = ureq::get(&url);
    req.set("Referer", "https://tenhou.net/");
    req.timeout_connect(10_000);
    proxy_from_env(&mut req, &url)?;

    let res = req.call();
    if res.ok() {
        Ok(res.into_string()?)
    } else {
        Err(anyhow!(
            "get tenhou log: {} {}",
            res.status(),
            res.status_text()
        ))
    }
}

pub fn mahjong_soul_log(log_id: &str) -> Result<String> {
    let mut ser = Serializer::new(String::new());
    ser.append_pair("id", log_id);
    let query = ser.finish();
    let url = format!("https://tensoul.herokuapp.com/convert?{}", query);

    let mut req = ureq::get(&url);
    req.timeout_connect(20_000);
    proxy_from_env(&mut req, &url)?;

    let res = req.call();
    if res.ok() {
        Ok(res.into_string()?)
    } else {
        Err(anyhow!(
            "get mahjong soul log: {} {}",
            res.status(),
            res.status_text()
        ))
    }
}

fn proxy_from_env(req: &mut ureq::Request, url: &str) -> Result<()> {
    if let Some(proxy_url) = env_proxy::for_url_str(&url).raw_value() {
        let proxy_str: String = proxy_url.chars().skip("http://".len()).collect();
        let proxy = ureq::Proxy::new(proxy_str).context("failed to parse proxy")?;
        req.set_proxy(proxy);
    }

    Ok(())
}
