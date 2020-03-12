mod download;
mod log;
mod metadata;
mod render;
mod review;
mod state;
mod tactics;
mod tehai;

use download::download_tenhou_log;
use metadata::Metadata;
use render::render;
use review::review;
use tactics::TacticsJson;

use std::env;
use std::ffi::OsString;
use std::fs::remove_file;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::result::Result as StdResult;

use anyhow::{Context, Result};
use clap::value_t_or_exit;
use clap::{App, Arg};
use convlog::tenhou;
use dunce::canonicalize;
use opener;
use serde_json;
use tee::TeeReader;
use tempfile::NamedTempFile;

static PKG_NAME: &str = env!("CARGO_PKG_NAME");
static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
static GIT_HASH: &str = env!("GIT_HASH");
static BUILD_DATE: &str = env!("BUILD_DATE");
static BUILD_PROFILE: &str = env!("BUILD_PROFILE");
static RUSTC_VERSION: &str = env!("RUSTC_VERSION");
static RUSTC_HOST: &str = env!("RUSTC_HOST");
static RUSTC_TARGET: &str = env!("RUSTC_TARGET");

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
                .required(true)
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
                    It is the number after \"&tw=\" in tenhou's log url",
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
                    If FILE is \"-\" or empty, read from stdin",
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
                    if --tenhou-id is specified, otherwise \"report.html\"",
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
                    Example: \"2019050417gm-0029-0000-4f2a8622\"",
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
                    If FILE is \"-\", write to stdout",
                ),
        )
        .arg(
            Arg::with_name("mjai-out")
                .long("mjai-out")
                .takes_value(true)
                .value_name("FILE")
                .help(
                    "Save the transformed mjai format log to FILE. \
                    If FILE is \"-\", write to stdout",
                ),
        )
        .arg(
            Arg::with_name("without-viewer")
                .long("without-viewer")
                .help("Do not include log viewer in the generated HTML report"),
        )
        .arg(
            Arg::with_name("no-open")
                .long("no-open")
                .help("Do not open the output file after finishing"),
        )
        .arg(
            Arg::with_name("no-review")
                .long("no-review")
                .help("Do not review at all. Only download and save files"),
        )
        .arg(
            Arg::with_name("akochan-dir")
                .short("d")
                .long("akochan-dir")
                .takes_value(true)
                .value_name("DIR")
                .help(
                    "Specify the directory of akochan. \
                    This will serves as the working directory of akochan process. \
                    Default value is the directory in which --akochan-exe is specified",
                ),
        )
        .arg(
            Arg::with_name("akochan-exe")
                .short("e")
                .long("akochan-exe")
                .takes_value(true)
                .value_name("EXE")
                .help(
                    "Specify the executable file of akochan. \
                    Default value \"akochan/system.exe\"",
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
                    Default value \"tactics.json\"",
                ),
        )
        .arg(
            Arg::with_name("pt")
                .long("pt")
                .takes_value(true)
                .value_name("ARRAY")
                .validator(|v| {
                    let arr = v
                        .split(',')
                        .map(|p| {
                            p.parse::<i32>()
                                .map_err(|err| format!("pt element must be a number: {}", err))
                        })
                        .collect::<Vec<StdResult<_, String>>>();

                    if arr.len() != 4 {
                        Err("pt must have exactly 4 elements".to_owned())
                    } else {
                        Ok(())
                    }
                })
                .help(
                    "Shortcut to override \"jun_pt\" in --tactics-config. \
                    Format: \"90,45,0,-135\"",
                ),
        )
        .arg(
            Arg::with_name("full")
                .short("f")
                .long("full")
                .help("Analyze every move, not only the different ones."),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Use verbose output"),
        )
        .get_matches();

    // get actor
    let actor: u8 = value_t_or_exit!(matches, "actor", u8);

    // load io specific options
    let arg_in_file = matches.value_of_os("in-file");
    let arg_out_file = matches.value_of_os("out-file");
    let arg_tenhou_id = matches.value_of("tenhou-id");
    let arg_tenhou_out = matches.value_of_os("tenhou-out");
    let arg_mjai_out = matches.value_of_os("mjai-out");

    // get log reader, can be from a file, from stdin, or from HTTP stream
    let log_reader: Box<dyn Read> = {
        if let Some(tenhou_id) = arg_tenhou_id {
            let log_stream = download_tenhou_log(tenhou_id)
                .with_context(|| format!("failed to download tenhou log ID={:?}", tenhou_id))?;

            // handle --tenhou-out
            if let Some(tenhou_out) = arg_tenhou_out {
                if tenhou_out != "-" {
                    let tenhou_out_file = File::create(tenhou_out).with_context(|| {
                        format!("failed to create download out file {:?}", tenhou_out)
                    })?;
                    Box::new(TeeReader::new(log_stream, tenhou_out_file))
                } else {
                    Box::new(TeeReader::new(log_stream, io::stdout()))
                }
            } else {
                Box::new(log_stream)
            }
        } else {
            match arg_in_file {
                Some(in_file_path) if in_file_path != "-" => {
                    let in_file = File::open(in_file_path).with_context(|| {
                        format!("failed to open tenhou log file {:?}", in_file_path)
                    })?;
                    let in_file_reader = BufReader::new(in_file);

                    Box::new(in_file_reader)
                }
                _ => Box::new(io::stdin()),
            }
        }
    };

    // parse tenhou log from reader
    let begin_parse_log = chrono::Local::now();
    log!("parsing tenhou log...");
    let raw_log: tenhou::RawLog =
        serde_json::from_reader(log_reader).context("failed to parse tenhou log")?;

    // clone the parsed raw log for possible reuse (split)
    let cloned_raw_log = if !matches.is_present("without-reviewer") {
        Some(raw_log.clone())
    } else {
        None
    };

    // convert from RawLog to Log.
    // it moves raw_log.
    let log = tenhou::Log::from(raw_log);

    // convert from tenhou::Log to Vec<mjai::Event>
    let begin_convert_log = chrono::Local::now();
    log!("converting to mjai events...");
    let events =
        convlog::tenhou_to_mjai(&log).context("failed to convert tenhou log into mjai format")?;

    // handle --mjai-out
    if let Some(mjai_out) = arg_mjai_out {
        let mut w: Box<dyn Write> = if mjai_out != "-" {
            let mjai_out_file = File::create(mjai_out)
                .with_context(|| format!("failed to create mjai out file {:?}", mjai_out))?;
            Box::from(mjai_out_file)
        } else {
            Box::from(io::stdout())
        };

        for event in &events {
            serde_json::to_writer(&mut w, event)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_out))?;
            writeln!(w)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_out))?;
        }
    }

    if matches.is_present("no-review") {
        return Ok(());
    }

    // get paths
    let akochan_exe = {
        let path = matches
            .value_of_os("akochan-exe")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut path = if let Ok(current_dir) = env::current_dir() {
                    current_dir
                } else {
                    PathBuf::from(".")
                };

                path.push("akochan");
                path.push("system.exe");

                path
            });

        canonicalize(&path)
            .with_context(|| format!("failed to canonicalize akochan_exe path {:?}", path))?
    };
    let akochan_dir = {
        let path = matches
            .value_of_os("akochan_dir")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut dir = akochan_exe.clone();
                dir.pop();
                dir
            });

        canonicalize(&path)
            .with_context(|| format!("failed to canonicalize akochan_dir path {:?}", path))?
    };
    let (tactics_file_path, tactics) = {
        let path = matches
            .value_of_os("tactics-config")
            .map(PathBuf::from)
            .unwrap_or_else(|| "tactics.json".into());

        let canon_path = canonicalize(&path)
            .with_context(|| format!("failed to canonicalize tactics_config path {:?}", path))?;

        // load tactics_config for metadata
        let tactics_file = File::open(&canon_path)
            .with_context(|| format!("failed to open tactics_config {:?}", canon_path))?;
        let tactics_file_reader = BufReader::new(tactics_file);

        let mut tactics_json: TacticsJson = serde_json::from_reader(tactics_file_reader)
            .with_context(|| format!("failed to parse tactics_config {:?}", canon_path))?;

        // opt-in pt
        if let Some(pt) = matches.value_of("pt") {
            tactics_json
                .tactics
                .jun_pt
                .iter_mut()
                .zip(pt.split(',').map(|p| p.parse::<i32>().unwrap()))
                .for_each(|(o, n)| *o = n);

            let mut tmp = NamedTempFile::new().context("failed to create temp file")?;
            serde_json::to_writer(&mut tmp, &tactics_json)
                .context("failed to write to temp file")?;

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

    log!("players: {:?}", log.names);
    log!("target: {}", log.names[actor as usize]);
    log!("start review, this may take serval minutes...");
    let full = matches.is_present("full");
    let verbose = matches.is_present("verbose");

    // do the review
    let begin_review = chrono::Local::now();
    let reviews = review(
        &akochan_exe,
        &akochan_dir,
        &tactics_file_path,
        full,
        &events,
        actor,
        verbose,
    )
    .context("failed to review log")?;

    // clean up
    if matches.is_present("pt") {
        remove_file(&tactics_file_path)
            .with_context(|| format!("failed to clean up temp file {:?}", tactics_file_path))?;
    }

    // determine whether the file can be opened after writing
    let opanable_file = match arg_out_file {
        Some(out_file_path) => {
            if out_file_path != "-" {
                Some(out_file_path.to_owned())
            } else {
                None
            }
        }
        _ => {
            if let Some(tenhou_id) = arg_tenhou_id {
                Some(OsString::from(format!("{}&tw={}.html", tenhou_id, actor)))
            } else {
                Some(OsString::from("report.html"))
            }
        }
    };

    // prepare output, can be a file or stdout
    let mut out: Box<dyn Write> = if let Some(out_file_path) = &opanable_file {
        let out_file = File::create(out_file_path)
            .with_context(|| format!("failed to create HTML report file {:?}", out_file_path))?;
        Box::new(out_file)
    } else {
        Box::new(io::stdout())
    };

    let now = chrono::Local::now();
    let parse_time = (begin_convert_log - begin_parse_log).to_std()?;
    let convert_time = (begin_review - begin_convert_log).to_std()?;
    let review_time = (now - begin_review).to_std()?;
    let meta = Metadata {
        pt: &tactics.jun_pt,
        game_length: &log.game_length.to_string(),
        parse_time,
        convert_time,
        review_time,
        tenhou_id: arg_tenhou_id,
        version: &format!("v{} ({})", PKG_VERSION, GIT_HASH),
    };

    // render the HTML report page
    log!("rendering output...");
    if let Some(l) = cloned_raw_log {
        render(&mut out, &reviews, actor, &meta, Some(&l.split_by_kyoku()))
    } else {
        render(&mut out, &reviews, actor, &meta, None)
    }
    .context("failed to render HTML report")?;

    // open the output page
    if !matches.is_present("no-open") {
        if let Some(out_file_path) = &opanable_file {
            opener::open(out_file_path).with_context(|| {
                format!(
                    "failed to open rendered HTML report file {:?}",
                    out_file_path
                )
            })?;
        }
    }

    log!("done");
    Ok(())
}
