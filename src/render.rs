use crate::opts::Engine;
use crate::review::Review;
use convlog::tenhou::{GameLength, RawPartialLog};
use convlog::Event;
use fluent_templates::FluentLoader;
use std::collections::HashMap;
use std::io::prelude::*;
use std::time::Duration;

use anyhow::Result;
use minify_html::{minify, Cfg};
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;
use tera::Tera;

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fn build_base_templates() -> Result<Tera> {
    let mut tera = Tera::default();
    tera.autoescape_on(vec![".tera", ".html"]);

    tera.register_function("kyoku_to_bakaze", kyoku_to_bakaze);
    tera.register_function("kyoku_to_kyoku_in_bakaze", kyoku_to_kyoku_in_bakaze);
    tera.register_function("pretty_round", pretty_round);

    tera.add_raw_templates([
        ("macros.tera", include_str!("../templates/macros.tera")),
        ("report.tera", include_str!("../templates/report.tera")),
        ("report.css", include_str!("../templates/report.css")),
        ("report.js", include_str!("../templates/report.js")),
        ("pai.svg", include_str!("../assets/pai.svg")),
    ])?;

    Ok(tera)
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct View<'a> {
    // metadata
    pub engine: Engine,
    // pub pt: [i32; 4],
    pub game_length: GameLength,
    pub log_id: Option<&'a str>,
    #[serde(with = "humantime_serde")]
    pub loading_time: Duration,
    #[serde(with = "humantime_serde")]
    pub review_time: Duration,
    pub show_rating: bool,
    pub version: &'a str,

    // review
    pub review: Review,
    pub player_id: u8,

    pub split_logs: Option<&'a [RawPartialLog<'a>]>,
    pub mjai_log: &'a [Event],

    pub lang: &'a str,
}

impl View<'_> {
    pub fn render<W>(&self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        let mut templates = build_base_templates()?;
        let lang_id = self.lang.parse()?;
        templates.register_function(
            "fluent",
            FluentLoader::new(&*LOCALES).with_default_lang(lang_id),
        );
        let ctx = tera::Context::from_serialize(self)?;
        let original = templates.render("report.tera", &ctx)?;

        let cfg = Cfg {
            keep_comments: true,
            minify_css: true,
            minify_js: true,
            ..Cfg::spec_compliant()
        };
        let out = minify(original.as_bytes(), &cfg);

        w.write_all(&out)?;
        Ok(())
    }
}

fn kyoku_to_bakaze(args: &HashMap<String, Value>) -> tera::Result<Value> {
    const BAKAZE: &[&str] = &["East", "South", "West", "North"];

    let kyoku = args
        .get("kyoku")
        .and_then(|p| p.as_u64())
        .ok_or_else(|| tera::Error::msg("missing or invalid argument `kyoku`"))?
        as usize;

    Ok(BAKAZE[kyoku / 4].into())
}

fn kyoku_to_kyoku_in_bakaze(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let kyoku = args
        .get("kyoku")
        .and_then(|p| p.as_u64())
        .ok_or_else(|| tera::Error::msg("missing or invalid argument `kyoku`"))?
        as usize;

    Ok((kyoku % 4 + 1).into())
}

fn pretty_round(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let num = args
        .get("num")
        .and_then(|n| n.as_f64())
        .ok_or_else(|| tera::Error::msg("missing or invalid argument `num`"))?;

    let prec = args.get("prec").and_then(|p| p.as_u64()).unwrap_or(5);
    let split = args.get("split").and_then(|p| p.as_bool()).unwrap_or(false);

    let multiplier = 10_f64.powi(prec as i32);
    let num = (num * multiplier).round() / multiplier;

    let s = format!("{num:.0$}", prec as usize);
    if !split {
        return Ok(Value::String(s));
    }
    let seps = s.split('.').map(|s| Value::String(s.to_owned())).collect();
    Ok(Value::Array(seps))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn template_compile() {
        let _ = &*LOCALES;
        build_base_templates().expect("failed to parse template");
    }
}
