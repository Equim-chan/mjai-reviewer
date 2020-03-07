use super::review::KyokuReview;
use anyhow::Result;
use convlog::tenhou::RawPartialLog;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::prelude::*;
use tera;
use tera::{Context, Tera, Value};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        tera.add_raw_templates(vec![
            ("macros.html", include_str!("../templates/macros.html")),
            ("pai_assets.html", include_str!("../res/pai_assets.html")),
            ("report.html", include_str!("../templates/report.html")),
        ])
        .expect("failed to parse template");

        tera.register_function("kyoku_to_string", kyoku_to_string);

        tera
    };
}

fn kyoku_to_string<'a>(args: &'a HashMap<String, Value>) -> tera::Result<Value> {
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

#[derive(Serialize)]
struct View<'a, 'b> {
    target_actor: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    splited_logs: Option<&'a [RawPartialLog<'a>]>,
    kyokus: &'b [KyokuReview],
}

pub fn render<'a, W>(
    w: &mut W,
    reviews: &[KyokuReview],
    target_actor: u8,
    splited_logs: Option<&'a [RawPartialLog<'a>]>,
) -> Result<()>
where
    W: Write,
{
    let view = View {
        target_actor,
        splited_logs,
        kyokus: reviews,
    };

    let ctx = Context::from_serialize(view)?;
    let result = TEMPLATES.render("report.html", &ctx)?;

    w.write_all(&result.as_bytes())?;

    Ok(())
}
