use crate::metadata::Metadata;
use crate::review::KyokuReview;

use std::collections::HashMap;
use std::convert::TryFrom;
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
    tera.register_function("pretty_round_2", pretty_round_2);
    tera.register_function("pretty_round_4", pretty_round_4);

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

fn kyoku_to_string_ja(args: &HashMap<String, Value>) -> tera::Result<Value> {
    const BAKAZE_KANJI: &[&str] = &["東", "南", "西", "北"];
    const NUM_KANJI: &[&str] = &["一", "二", "三", "四"];

    let kyoku = if let Some(Value::Number(num)) = args.get("kyoku") {
        usize::try_from(num.as_u64().unwrap_or(0)).unwrap_or(0)
    } else {
        0
    };

    let honba = if let Some(Value::Number(num)) = args.get("honba") {
        usize::try_from(num.as_u64().unwrap_or(0)).unwrap_or(0)
    } else {
        0
    };

    let ret = BAKAZE_KANJI[kyoku / 4].to_owned() + NUM_KANJI[kyoku % 4] + "局";

    if honba == 0 {
        Ok(Value::String(ret))
    } else {
        Ok(Value::String(ret + " " + &honba.to_string() + " 本場"))
    }
}

fn kyoku_to_string_en(args: &HashMap<String, Value>) -> tera::Result<Value> {
    const BAKAZE_ENG: &[&str] = &["East", "South", "West", "North"];
    const NUM_ENG: &[&str] = &["1", "2", "3", "4"];

    let kyoku = if let Some(Value::Number(num)) = args.get("kyoku") {
        usize::try_from(num.as_u64().unwrap_or(0)).unwrap_or(0)
    } else {
        0
    };

    let honba = if let Some(Value::Number(num)) = args.get("honba") {
        usize::try_from(num.as_u64().unwrap_or(0)).unwrap_or(0)
    } else {
        0
    };

    let ret = BAKAZE_ENG[kyoku / 4].to_owned() + " " + NUM_ENG[kyoku % 4];

    if honba == 0 {
        Ok(Value::String(ret))
    } else {
        Ok(Value::String(ret + "-" + &honba.to_string()))
    }
}

fn pretty_round_2(args: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(Value::Number(num)) = args.get("num") {
        if let Some(f) = num.as_f64() {
            let n = (f * 1e4).round() / 1e4;
            let s = format!("{:.02}", n);

            return Ok(Value::String(s));
        }
    }

    Ok(Value::Null)
}

fn pretty_round_4(args: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(Value::Number(num)) = args.get("num") {
        if let Some(f) = num.as_f64() {
            let n = (f * 1e4).round() / 1e4;
            let s = format!("{:.04}", n);

            return Ok(Value::String(s));
        }
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
}

impl<'a, L> View<'a, L>
where
    L: AsRef<[RawPartialLog<'a>]> + Serialize,
{
    #[inline]
    pub fn new(
        kyoku_reviews: &'a [KyokuReview],
        target_actor: u8,
        splited_logs: Option<L>,
        metadata: &'a Metadata<'a>,
        lang: Language,
    ) -> Self {
        Self {
            kyokus: kyoku_reviews,
            target_actor,
            splited_logs,
            metadata,
            lang,
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
        w.write_all(&result.as_bytes())?;

        Ok(())
    }
}
