mod download;
mod log;
mod log_source;
mod metadata;
mod raw_log_ext;
mod render;
mod report_output;
mod review;
mod state;
mod tactics;
mod tehai;

use crate::render::Layout;

use self::log_source::LogSource;
use self::metadata::Metadata;
use self::raw_log_ext::RawLogExt;
use self::render::{Language, View};
use self::report_output::ReportOutput;
use self::review::review;
use self::review::ReviewArgs;
use self::tactics::TacticsJson;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use anyhow::{Context, Result};
use clap::{App, Arg};
use convlog::tenhou;
use dunce::canonicalize;
use serde_json as json;
use tempfile::NamedTempFile;
use url::Url;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const GIT_HASH: &str = env!("GIT_HASH");
const BUILD_DATE: &str = env!("BUILD_DATE");
const BUILD_PROFILE: &str = env!("BUILD_PROFILE");
const RUSTC_VERSION: &str = env!("RUSTC_VERSION");
const RUSTC_HOST: &str = env!("RUSTC_HOST");
const RUSTC_TARGET: &str = env!("RUSTC_TARGET");

fn main() -> Result<()> {
    let matches = App::new(PKG_NAME)
        .about(PKG_DESCRIPTION)
        .long_version(&*format!(
            "v{} ({}) {} {} build\n\
            [{}] {}/{}\n",
            PKG_VERSION,
            GIT_HASH,
            BUILD_DATE,
            BUILD_PROFILE,
            RUSTC_VERSION,
            RUSTC_HOST,
            RUSTC_TARGET,
        ))
        .arg(
            Arg::with_name("actor")
                .short("a")
                .long("actor")
                .takes_value(true)
                .value_name("INDEX")
                .validator(|v| {
                    let num: u8 = v
                        .parse()
                        .map_err(|err| format!("INDEX must be a number: {}", err))?;

                    if num > 3 {
                        Err(format!("INDEX must be within 0~3, got {}", num))
                    } else {
                        Ok(())
                    }
                })
                .help(
                    "Specify the actor to review. \
                    It is the number after \"&tw=\" in tenhou's log url.",
                ),
        )
        .arg(
            Arg::with_name("actor-name")
                .long("actor-name")
                .takes_value(true)
                .value_name("ACTOR_NAME")
                .help(
                    "Specify the actor name to review \
                    when --in-file is specified and --actor is not specified",
                ),
        )
        .arg(
            Arg::with_name("kyokus")
                .short("k")
                .long("kyokus")
                .takes_value(true)
                .value_name("LIST")
                .help(
                    "Specify kyokus to review. If LIST is empty, review all kyokus. \
                    Format: \"E1,E4,S3.1\".",
                ),
        )
        .arg(
            Arg::with_name("in-file")
                .short("i")
                .long("in-file")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Specify a tenhou.net/6 format log file to review. \
                    If FILE is \"-\" or empty, read from stdin.",
                ),
        )
        .arg(
            Arg::with_name("out-file")
                .short("o")
                .long("out-file")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Specify the output file for generated HTML report. \
                    If FILE is \"-\", write to stdout; \
                    if FILE is empty, write to \"{tenhou_id}&tw={actor}.html\" \
                    if --tenhou-id is specified, otherwise \"report.html\".",
                ),
        )
        .arg(
            Arg::with_name("tenhou-id")
                .short("t")
                .long("tenhou-id")
                .takes_value(true)
                .value_name("ID")
                .help(
                    "Specify a Tenhou log ID to review, overriding --in-file. \
                    Example: \"2019050417gm-0029-0000-4f2a8622\".",
                ),
        )
        .arg(
            Arg::with_name("mjsoul-id")
                .short("m")
                .long("mjsoul-id")
                .takes_value(true)
                .value_name("ID")
                .help(
                    "Specify a Mahjong Soul log ID to review. \
                    Example: \"200417-e1f9e08d-487f-4333-989f-34be08b943c7\".",
                ),
        )
        .arg(
            Arg::with_name("tenhou-out")
                .long("tenhou-out")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Save the downloaded tenhou.net/6 format log to FILE \
                    when --tenhou-id is specified. \
                    If FILE is \"-\", write to stdout.",
                ),
        )
        .arg(
            Arg::with_name("mjai-out")
                .long("mjai-out")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Save the transformed mjai format log to FILE. \
                    If FILE is \"-\", write to stdout.",
                ),
        )
        .arg(
            Arg::with_name("tenhou-ids-file")
                .long("tenhou-ids-file")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Specify a file of Tenhou log ID list to convert to mjai format, \
                    implying --no-review.",
                ),
        )
        .arg(
            Arg::with_name("out-dir")
                .long("out-dir")
                .takes_value(true)
                .value_name("DIR")
                .help(
                    "Specify a directory to save the output for mjai logs. \
                    If DIR is empty, defaults to \".\".",
                ),
        )
        .arg(
            Arg::with_name("without-viewer")
                .long("without-viewer")
                .help("Do not include log viewer in the generated HTML report."),
        )
        .arg(
            Arg::with_name("anonymous")
                .long("anonymous")
                .help("Do not include player names."),
        )
        .arg(
            Arg::with_name("no-open")
                .long("no-open")
                .help("Do not open the output file in browser after finishing."),
        )
        .arg(
            Arg::with_name("no-review")
                .long("no-review")
                .help("Do not review at all. Only download and save files."),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .help("Output review result in JSON instead of HTML."),
        )
        .arg(
            Arg::with_name("akochan-dir")
                .short("d")
                .long("akochan-dir")
                .takes_value(true)
                .value_name("DIR")
                .help(
                    "Specify the directory of akochan. \
                    This will serve as the working directory of akochan process. \
                    Default value \"akochan\".",
                ),
        )
        .arg(
            Arg::with_name("tactics-config")
                .short("c")
                .long("tactics-config")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Specify the tactics config file for akochan. \
                    Default value \"tactics.json\".",
                ),
        )
        .arg(
            Arg::with_name("pt")
                .long("pt")
                .takes_value(true)
                .value_name("LIST")
                .validator(|v| {
                    let list = v.split(',').map(|p| {
                        p.parse::<i32>()
                            .map_err(|err| format!("pt element must be a number: {}", err))
                    });

                    if list.count() != 4 {
                        Err("pt must have exactly 4 elements".to_owned())
                    } else {
                        Ok(())
                    }
                })
                .help(
                    "Shortcut to override \"jun_pt\" in --tactics-config. \
                    Format: \"90,45,0,-135\".",
                ),
        )
        .arg(
            Arg::with_name("use-placement-ev")
                .short("e")
                .long("use-placement-ev")
                .help(
                    "Use final placement EV instead of pt EV. \
                    This will override --pt and \"jun_pt\" in --tactics-config.",
                ),
        )
        .arg(
            Arg::with_name("deviation-threshold")
                .short("n")
                .long("deviation-threshold")
                .takes_value(true)
                .value_name("THRESHOLD")
                .validator(|v| {
                    v.parse::<f64>()
                        .map(|_| ())
                        .map_err(|err| format!("THRESHOLD must be a number: {}", err))
                })
                .help(
                    "THRESHOLD is an absolute value that the reviewer will ignore all \
                    problematic moves whose EVs are within the range of \
                    [best EV - THRESHOLD, best EV]. \
                    This option is effective under both pt and placement EV mode. \
                    It is recommended to use it with --use-placement-ev where the reward \
                    distribution is fixed and even. \
                    Reference value: 0.05 when using pt and 0.001 when using placement. \
                    Default value: \"0.001\".",
                ),
        )
        .arg(
            Arg::with_name("lang")
                .long("lang")
                .takes_value(true)
                .value_name("LANG")
                .help(
                    "Set the language for the rendered report page. \
                    Default value \"ja\". \
                    Supported languages: ja, en.",
                )
                .validator(|v| match v.as_str() {
                    "ja" | "en" => Ok(()),
                    _ => Err(format!("unsupported language {}", v)),
                }),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Use verbose output."),
        )
        .arg(
            Arg::with_name("layout")
                .long("layout")
                .takes_value(true)
                .value_name("LAYOUT")
                .help(
                    "Set the layout for the rendered report page. \
                    Default value \"vertical\". \
                    Supported layout: vertical, v, horizontal, h.",
                )
                .validator(|v| match v.as_str() {
                    "v" | "vertical" | "h" | "horizontal" => Ok(()),
                    _ => Err(format!("unsupported layout {}", v)),
                }),
        )
        .arg(Arg::with_name("URL").help("Tenhou or Mahjong Soul log URL."))
        .get_matches();

    // load options
    let arg_in_file = matches.value_of_os("in-file");
    let arg_out_file = matches.value_of_os("out-file");
    let arg_tenhou_id = matches.value_of("tenhou-id").map(String::from);
    let arg_mjsoul_id = matches.value_of("mjsoul-id").map(String::from);
    let arg_tenhou_out = matches.value_of_os("tenhou-out");
    let arg_mjai_out = matches.value_of_os("mjai-out");
    let arg_tenhou_ids_file = matches.value_of_os("tenhou-ids-file");
    let arg_out_dir = matches.value_of_os("out-dir");
    let arg_akochan_dir = matches.value_of_os("akochan-dir");
    let arg_tactics_config = matches.value_of_os("tactics-config");
    let arg_actor: Option<u8> = matches.value_of("actor").map(|p| p.parse().unwrap());
    let arg_actor_name: Option<String> = matches.value_of("actor-name").map(String::from);
    let arg_pt = matches.value_of("pt");
    let arg_kyokus = matches.value_of("kyokus");
    let arg_use_placement_ev = matches.is_present("use-placement-ev");
    let arg_without_viewer = matches.is_present("without-viewer");
    let arg_anonymous = matches.is_present("anonymous");
    let arg_no_open = matches.is_present("no-open");
    let arg_no_review = matches.is_present("no-review");
    let arg_json = matches.is_present("json");
    let arg_deviation_threshold = matches
        .value_of("deviation-threshold")
        .map(|v| v.parse().unwrap())
        .unwrap_or(0.001);
    let arg_lang = matches.value_of("lang");
    let arg_verbose = matches.is_present("verbose");
    let arg_url = matches.value_of("URL");

    let layout = match matches.value_of("layout") {
        None => Layout::Vertical,
        Some(arg_layout) => match arg_layout {
            "h" | "horizontal" => Layout::Horizontal,
            "v" | "vertical" => Layout::Vertical,
            _ => unreachable!(),
        },
    };

    if let Some(tenhou_ids_file) = arg_tenhou_ids_file {
        let out_dir_name = arg_out_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        return batch_download(&out_dir_name, Path::new(tenhou_ids_file));
    }

    // sometimes the log URL contains the actor info
    let mut actor_opt = arg_actor;

    let log_source = if let Some(filename) = arg_in_file {
        if filename == "-" {
            LogSource::Stdin
        } else {
            LogSource::File(filename.to_owned())
        }
    } else if let Some(id) = arg_tenhou_id {
        LogSource::Tenhou(id)
    } else if let Some(raw_id) = arg_mjsoul_id {
        LogSource::mjsoul_full_id_with_deobfuse(&raw_id)
    } else if let Some(url) = arg_url {
        let u = Url::parse(url).context("failed to parse URL")?;
        let host = u.host_str().context("url does not have host")?;
        match host {
            "tenhou.net" => {
                let (mut log, mut tw) = (None, None);
                for (k, v) in u.query_pairs() {
                    match &*k {
                        "log" => log = Some(v.into_owned()),
                        "tw" => {
                            let num: u8 = v.parse().context("\"tw\" must be a number")?;
                            if num > 3 {
                                return Err(anyhow!("\"tw\" must be within 0~3, got {}", num));
                            }

                            tw = Some(num);
                        }
                        _ => continue,
                    };

                    if log.is_some() && tw.is_some() {
                        break;
                    }
                }

                actor_opt = actor_opt.or(tw).or(Some(0));
                match log {
                    Some(id) => LogSource::Tenhou(id),
                    None => return Err(anyhow!("tenhou log ID not found in URL {}", url)),
                }
            }

            "game.mahjongsoul.com" /* JP */
            | "mahjongsoul.game.yo-star.com" /* US */
            | "game.maj-soul.com" /* steam */
            | "www.majsoul.com" /* legacy CN */
            | "majsoul.union-game.com" /* legacy CN */ => {
                let mut paipu = None;
                for (k, v) in u.query_pairs() {
                    if k == "paipu" {
                        paipu = Some(v.into_owned());
                        break;
                    }
                }

                match paipu {
                    Some(raw_id) => {
                        LogSource::mjsoul_full_id_with_deobfuse(&raw_id)
                    }
                    None => return Err(anyhow!("mahjong soul log ID not found in URL {}", url)),
                }
            }

            _ => {
                return Err(anyhow!(
                    "specified url is neither from tenhou nor mahjong soul"
                ))
            }
        }
    } else {
        LogSource::Stdin
    };

    // handle --tenhou-out
    let tenhou_out = arg_tenhou_out
        .map(|filename| -> Result<(Box<dyn Write>, _)> {
            if filename == "-" {
                Ok((Box::new(io::stdout()), "stdout".to_owned().into()))
            } else {
                let file = File::create(filename).with_context(|| {
                    format!("failed to create tenhou output file {:?}", filename)
                })?;
                Ok((Box::new(file), filename.to_owned()))
            }
        })
        .transpose()?;

    // download and parse tenhou.net/6 log
    let mut raw_log: tenhou::RawLog = match &log_source {
        LogSource::Tenhou(id) => {
            let body = download::tenhou_log(id)
                .with_context(|| format!("failed to download tenhou log {}", id))?;
            if let Some((mut writer, filename)) = tenhou_out {
                writer.write_all(body.as_bytes()).with_context(|| {
                    format!("failed to write downloaded tenhou log to {:?}", filename)
                })?;
            }

            json::from_str(&body).context("failed to parse tenhou.net/6 log")?
        }
        LogSource::MahjongSoul(id) => {
            let body = download::mahjong_soul_log(id)
                .with_context(|| format!("failed to download mahjong soul log {}", id))?;
            if let Some((mut writer, filename)) = tenhou_out {
                writer.write_all(body.as_bytes()).with_context(|| {
                    format!("failed to write downloaded tenhou log to {:?}", filename)
                })?;
            }

            let val: RawLogExt =
                json::from_str(&body).context("failed to parse tenhou.net/6 log")?;

            actor_opt = actor_opt.or(val.target_actor);
            val.raw_log
        }
        LogSource::File(filename) => {
            let mut file = File::open(&filename)
                .with_context(|| format!("failed to open tenhou.net/6 log file {:?}", filename))?;
            let mut body = String::new();
            file.read_to_string(&mut body)?;

            json::from_str(&body).context("failed to parse tenhou.net/6 log")?
        }
        LogSource::Stdin => {
            let stdin = io::stdin();
            let handle = stdin.lock();
            json::from_reader(handle).context("failed to parse tenhou.net/6 log")?
        }
    };

    // Try to match the name from arg
    if actor_opt.is_none() {
        if let Some(actor_name) = arg_actor_name {
            for (idx, n) in raw_log.get_names().iter().enumerate() {
                if *n == actor_name {
                    actor_opt = Some(idx as u8)
                }
            }
        }
    }

    // apply filters
    if arg_anonymous {
        raw_log.hide_names();
    }
    if let Some(expr) = arg_kyokus {
        let filter = expr.parse().context("failed to parse kyoku filter")?;
        raw_log.filter_kyokus(&filter);
        if raw_log.is_empty() {
            return Err(anyhow!("no kyoku to review (invalid filter?)"));
        }
    }

    // clone the parsed raw log for possible reuse (split)
    //
    // See https://manishearth.github.io/blog/2017/04/13/prolonging-temporaries-in-rust/
    // for the technique of extending the lifetime of temp var here.
    let cloned_raw_log;
    let splitted_raw_logs = if !arg_without_viewer {
        cloned_raw_log = raw_log.clone();
        Some(cloned_raw_log.split_by_kyoku())
    } else {
        None
    };

    // convert from RawLog to Log.
    // it moves raw_log.
    let log = tenhou::Log::from(raw_log);

    // convert from tenhou::Log to Vec<mjai::Event>
    let begin_convert_log = chrono::Local::now();
    log!("converting to mjai events...");
    let events = convlog::tenhou_to_mjai(&log)
        .context("failed to convert tenhou.net/6 log into mjai format")?;

    // handle --mjai-out
    if let Some(mjai_out) = arg_mjai_out {
        let mut w: Box<dyn Write> = if mjai_out == "-" {
            Box::from(io::stdout())
        } else {
            let mjai_out_file = File::create(mjai_out)
                .with_context(|| format!("failed to create mjai out file {:?}", mjai_out))?;
            Box::from(mjai_out_file)
        };

        for event in &events {
            let to_write = json::to_string(event).context("failed to serialize")?;
            writeln!(w, "{}", to_write)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_out))?;
        }
    }

    if arg_no_review {
        return Ok(());
    }

    // get actor
    let actor = actor_opt.context("actor is required")?;
    if actor > 3 {
        // just in case
        return Err(anyhow!("must be within 0~3, got {}", actor));
    }

    // get paths
    let akochan_dir = {
        let path = arg_akochan_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("akochan"));

        canonicalize(&path)
            .with_context(|| format!("failed to canonicalize akochan_dir path {:?}", path))?
    };
    let akochan_exe = canonicalize(
        [&*akochan_dir, "system.exe".as_ref()]
            .iter()
            .collect::<PathBuf>(),
    )
    .context("failed to canonicalize akochan_exe path")?;
    let (tactics_file_path, tactics) = {
        let path = arg_tactics_config
            .map(PathBuf::from)
            .unwrap_or_else(|| "tactics.json".into());

        let canon_path = canonicalize(&path)
            .with_context(|| format!("failed to canonicalize tactics_config path {:?}", path))?;

        // load tactics_config for metadata
        let tactics_file = File::open(&canon_path)
            .with_context(|| format!("failed to open tactics_config {:?}", canon_path))?;
        let tactics_file_reader = BufReader::new(tactics_file);

        let mut tactics_json: TacticsJson = json::from_reader(tactics_file_reader)
            .with_context(|| format!("failed to parse tactics_config {:?}", canon_path))?;

        // opt-in pt
        let pt_opt = if arg_use_placement_ev {
            Some(vec![-1, -2, -3, -4])
        } else {
            arg_pt.map(|pt| pt.split(',').map(|p| p.parse::<i32>().unwrap()).collect())
        };

        if let Some(pt) = pt_opt {
            tactics_json
                .tactics
                .jun_pt
                .iter_mut()
                .zip(pt)
                .for_each(|(o, n)| *o = n);

            let mut tmp = NamedTempFile::new().context("failed to create temp file")?;
            json::to_writer(&mut tmp, &tactics_json).context("failed to write to temp file")?;

            let tmp_path = tmp
                .into_temp_path()
                .keep()
                .context("failed to keep temp file")?;
            let canon_tmp_path = canonicalize(&tmp_path)
                .with_context(|| format!("failed to canonicalize temp file path {:?}", tmp_path))?;

            (canon_tmp_path, tactics_json.tactics)
        } else {
            (canon_path, tactics_json.tactics)
        }
    };

    log!("players: {}", log.names.join(", "));
    log!("target: {}", log.names[actor as usize]);
    log!("review has started, this may take several minutes...");

    // do the review
    let begin_review = chrono::Local::now();
    let review_args = ReviewArgs {
        akochan_exe: &akochan_exe,
        akochan_dir: &akochan_dir,
        tactics_config: &tactics_file_path,
        events: &events,
        target_actor: actor,
        deviation_threshold: arg_deviation_threshold,
        verbose: arg_verbose,
    };
    let review_result = review(&review_args).context("failed to review log")?;

    // clean up temp file
    if arg_pt.is_some() {
        fs::remove_file(&tactics_file_path)
            .with_context(|| format!("failed to clean up temp file {:?}", tactics_file_path))?;
    }

    // determine language
    let lang = match arg_lang {
        Some("ja") | None => Language::Japanese,
        Some("en") => Language::English,
        _ => unreachable!(),
    };

    // determine output file
    let out = if let Some(filename) = arg_out_file {
        if filename == "-" {
            ReportOutput::Stdout
        } else {
            ReportOutput::File(filename.to_owned())
        }
    } else {
        let suffix = if arg_json { ".json" } else { ".html" };
        let mut filename = log_source.default_output_filename(actor);
        filename.push(suffix);
        ReportOutput::File(filename)
    };

    // prepare output, can be a file or stdout
    let mut out_write: Box<dyn Write> = match &out {
        ReportOutput::File(filename) => Box::new(
            File::create(&filename)
                .with_context(|| format!("failed to create output report file {:?}", filename))?,
        ),
        ReportOutput::Stdout => Box::new(io::stdout()),
    };

    let now = chrono::Local::now();
    let loading_time = (begin_review - begin_convert_log).to_std()?;
    let review_time = (now - begin_review).to_std()?;
    let meta = Metadata {
        pt: &tactics.jun_pt,
        game_length: &log.game_length.to_string(),
        loading_time,
        review_time,
        log_id: if arg_anonymous {
            None
        } else {
            log_source.log_id()
        },
        use_placement_ev: arg_use_placement_ev,
        deviation_threshold: arg_deviation_threshold,
        total_reviewed: review_result.total_reviewed,
        total_tolerated: review_result.total_tolerated,
        total_problems: review_result.total_problems,
        score: review_result.score,
        version: &format!("v{} ({})", PKG_VERSION, GIT_HASH),
    };

    // render the HTML report page or JSON
    let view = View::new(
        &review_result.kyokus,
        actor,
        splitted_raw_logs,
        &meta,
        lang,
        layout,
    );
    if arg_json {
        log!("writing output...");
        json::to_writer(&mut out_write, &view).context("failed to write JSON result")?;
    } else {
        log!("rendering output...");
        view.render(&mut out_write)
            .context("failed to render HTML report")?;
    }

    // open the output page
    if !arg_json && !arg_no_open {
        if let ReportOutput::File(filepath) = out {
            opener::open(&filepath).with_context(|| {
                format!("failed to open rendered HTML report file {:?}", filepath)
            })?;
        }
    }

    log!("done");
    Ok(())
}

fn batch_download(out_dir_name: &Path, tenhou_ids_file: &Path) -> Result<()> {
    fs::create_dir_all(&out_dir_name)
        .with_context(|| format!("failed to create {:?}", out_dir_name))?;

    log!("tenhou_ids_file: {:?}", tenhou_ids_file);

    for line in BufReader::new(File::open(tenhou_ids_file)?).lines() {
        let tenhou_id = line?;

        log!("downloading tenhou log {} ...", tenhou_id);
        let body = download::tenhou_log(&tenhou_id)
            .with_context(|| format!("failed to download tenhou log ID={:?}", tenhou_id))?;

        log!("parsing tenhou log {} ...", tenhou_id);
        let raw_log: tenhou::RawLog =
            json::from_str(&body).context("failed to parse tenhou log")?;
        let log = tenhou::Log::from(raw_log);

        log!("converting to mjai events...");
        let events = convlog::tenhou_to_mjai(&log)
            .context("failed to convert tenhou log into mjai format")?;

        let mjai_out = {
            let mut p = out_dir_name.to_owned();
            p.push(tenhou_id + ".json");
            p
        };
        let mut mjai_out_file = File::create(&mjai_out)
            .with_context(|| format!("failed to create mjai out file {:?}", mjai_out))?;

        for event in &events {
            let to_write = json::to_string(event).context("failed to serialize")?;
            writeln!(mjai_out_file, "{}", to_write)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_out))?;
        }
    }

    Ok(())
}
