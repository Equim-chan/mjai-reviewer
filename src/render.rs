use super::metadata::Metadata;
use super::review::KyokuReview;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::prelude::*;

use anyhow::{Context, Result};
use convlog::tenhou::RawPartialLog;
use lazy_static::lazy_static;
use serde::Serialize;
use tera;
use tera::{Tera, Value};

lazy_static! {
    static ref TEMPLATES: Tera = {
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
    };
}

fn kyoku_to_string(args: &HashMap<String, Value>) -> tera::Result<Value> {
    static BAKAZE_KANJI: &[&str] = &["東", "南", "西", "北"];
    static NUM_KANJI: &[&str] = &["一", "二", "三", "四"];

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

            // let parts: Vec<_> = s.split('.').collect();
            // let left = parts[0];
            // let right = parts[1]
            //     .as_bytes()
            //     .chunks(3)
            //     .map(|p| unsafe { std::str::from_utf8_unchecked(p) })
            //     .collect::<Vec<_>>()
            //     .join(" ");
            // let ret = format!("{}.{}", left, right);

            return Ok(Value::String(s));
        }
    }

    Ok(Value::Null)
}

#[derive(Serialize)]
struct View<'a> {
    kyokus: &'a [KyokuReview],
    target_actor: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    splited_logs: Option<&'a [RawPartialLog<'a>]>,
    metadata: &'a Metadata<'a>,
}

pub fn render<'a, W>(
    w: &mut W,
    reviews: &[KyokuReview],
    target_actor: u8,
    metadata: &Metadata,
    splited_logs: Option<&'a [RawPartialLog<'a>]>,
) -> Result<()>
where
    W: Write,
{
    let view = View {
        kyokus: reviews,
        target_actor,
        splited_logs,
        metadata,
    };

    let ctx = tera::Context::from_serialize(&view)?;
    let result =
        TEMPLATES.render("report.html", &ctx).with_context(|| {
            match serde_json::to_string(&view) {
                Ok(json_string) => format!("with values: {}", json_string),
                Err(err) => format!("even serializations failed: {}", err),
            }
        })?;
    w.write_all(&result.as_bytes())?;

    Ok(())
}
