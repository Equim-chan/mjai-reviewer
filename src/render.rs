use crate::metadata::Metadata;
use crate::review::KyokuReview;
use std::collections::HashMap;
use std::io::prelude::*;

use anyhow::{Context, Result};
use convlog::tenhou::RawPartialLog;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json as json;
use tera::{Tera, Value};

static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.register_function("kyoku_to_string_ja", kyoku_to_string_ja);
    tera.register_function("kyoku_to_string_en", kyoku_to_string_en);
    tera.register_function("pretty_round", pretty_round);

    tera.add_raw_templates(vec![
        ("macros.html", include_str!("../templates/macros.html")),
        ("pai.svg", include_str!("../assets/pai.svg")),
        ("report.css", include_str!("../templates/report.css")),
        ("report.html", include_str!("../templates/report.html")),
    ])
    .expect("failed to parse template");

    tera
});

#[derive(Serialize)]
pub enum Language {
    // The string is used in html lang attribute, as per BCP47.
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "en")]
    English,
}

#[derive(Serialize)]
pub enum Layout {
    #[serde(rename="horizontal")]
    Horizontal,
    #[serde(rename="vertical")]
    Vertical
}

#[allow(clippy::unnecessary_wraps)]
fn kyoku_to_string_ja(args: &HashMap<String, Value>) -> tera::Result<Value> {
    const BAKAZE_KANJI: &[&str] = &["東", "南", "西", "北"];
    const NUM_KANJI: &[&str] = &["一", "二", "三", "四"];

    let kyoku = args.get("kyoku").and_then(|p| p.as_u64()).unwrap_or(0) as usize;
    let honba = args.get("honba").and_then(|p| p.as_u64()).unwrap_or(0) as usize;

    let s = if honba == 0 {
        format!("{}{}局", BAKAZE_KANJI[kyoku / 4], NUM_KANJI[kyoku % 4])
    } else {
        format!(
            "{}{}局 {} 本場",
            BAKAZE_KANJI[kyoku / 4],
            NUM_KANJI[kyoku % 4],
            honba,
        )
    };
    Ok(Value::String(s))
}

#[allow(clippy::unnecessary_wraps)]
fn kyoku_to_string_en(args: &HashMap<String, Value>) -> tera::Result<Value> {
    const BAKAZE_ENG: &[&str] = &["East", "South", "West", "North"];
    const NUM_ENG: &[&str] = &["1", "2", "3", "4"];

    let kyoku = args.get("kyoku").and_then(|p| p.as_u64()).unwrap_or(0) as usize;
    let honba = args.get("honba").and_then(|p| p.as_u64()).unwrap_or(0) as usize;

    let s = if honba == 0 {
        format!("{} {}", BAKAZE_ENG[kyoku / 4], NUM_ENG[kyoku % 4])
    } else {
        format!("{} {}-{}", BAKAZE_ENG[kyoku / 4], NUM_ENG[kyoku % 4], honba)
    };
    Ok(Value::String(s))
}

#[allow(clippy::unnecessary_wraps)]
fn pretty_round(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let prec = args.get("prec").and_then(|p| p.as_u64()).unwrap_or(5);

    if let Some(num) = args.get("num").and_then(|n| n.as_f64()) {
        let pow = (10usize).pow(prec as u32) as f64;
        let f = (num * pow).round() / pow;
        let s = format!("{:.1$}", f, prec as usize);

        return Ok(Value::String(s));
    }

    Ok(Value::Null)
}

#[derive(Serialize)]
pub struct View<'a, L>
where
    L: AsRef<[RawPartialLog<'a>]> + Serialize,
{
    kyokus: &'a [KyokuReview],
    target_actor: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    splited_logs: Option<L>,
    metadata: &'a Metadata<'a>,
    lang: Language,
    layout: Layout,
}

impl<'a, L> View<'a, L>
where
    L: AsRef<[RawPartialLog<'a>]> + Serialize,
{
    #[inline]
    pub fn new(
        kyoku_reviews: &'a [KyokuReview],
        target_actor: u8,
        splitted_logs: Option<L>,
        metadata: &'a Metadata<'a>,
        lang: Language,
        layout: Layout
    ) -> Self {
        Self {
            kyokus: kyoku_reviews,
            target_actor,
            splited_logs: splitted_logs,
            metadata,
            lang,
            layout,
        }
    }

    pub fn render<W>(&self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        let ctx = tera::Context::from_serialize(&self)?;
        let result =
            TEMPLATES.render("report.html", &ctx).with_context(|| {
                match json::to_string(&self) {
                    Ok(json_string) => format!("with values: {}", json_string),
                    Err(err) => format!("even serializations failed: {}", err),
                }
            })?;
        w.write_all(result.as_bytes())?;

        Ok(())
    }
}
