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
use tera;
use tera::{Tera, Value};

static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.register_function("kyoku_to_string", kyoku_to_string);
    tera.register_function("pretty_round", pretty_round);

    tera.add_raw_templates(vec![
        ("macros.html", include_str!("../templates/macros.html")),
        ("pai.svg", include_str!("../assets/pai.svg")),
        ("report.html", include_str!("../templates/report.html")),
    ])
    .expect("failed to parse template");

    tera
});

fn kyoku_to_string(args: &HashMap<String, Value>) -> tera::Result<Value> {
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

fn pretty_round(args: &HashMap<String, Value>) -> tera::Result<Value> {
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
}

impl<'a, L> View<'a, L>
where
    L: AsRef<[RawPartialLog<'a>]> + Serialize,
{
    #[inline]
    pub fn new(
        kyoku_reviews: &'a [KyokuReview],
        target_actor: u8,
        metadata: &'a Metadata<'a>,
        splited_logs: Option<L>,
    ) -> Self {
        Self {
            kyokus: kyoku_reviews,
            target_actor,
            splited_logs,
            metadata,
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
