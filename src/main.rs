mod download;
mod log;
mod metadata;
mod render;
mod review;
mod tactics;

use anyhow::{Context, Result};
use clap::{App, Arg};
use convlog::tenhou;
use download::download_tenhou_log;
use dunce::canonicalize;
use metadata::Metadata;
use opener;
use render::render;
use review::review;
use serde_json;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use tactics::{Tactics, TacticsJson};
use tee::TeeReader;

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
                .help("Specify the actor to review"),
        )
        .arg(
            Arg::with_name("in-file")
                .short("i")
                .long("in-file")
                .takes_value(true)
                .value_name("FILE")
                .help("Specify a tenhou.net/6 format log file to review. If FILE is \"-\" or empty, read from stdin"),
        )
        .arg(
            Arg::with_name("out-file")
                .short("o")
                .long("out-file")
                .takes_value(true)
                .value_name("FILE")
                .help("Specify the output file for generated HTML report. If FILE is \"-\" or empty, write to stdout"),
        )
        .arg(
            Arg::with_name("tenhou-id")
                .short("t")
                .long("tenhou-id")
                .takes_value(true)
                .value_name("ID")
                .help("Specify a Tenhou log ID to review, overriding --in-file. For example: 2019050417gm-0029-0000-4f2a8622"),
        )
        .arg(
            Arg::with_name("tenhou-out")
                .long("tenhou-out")
                .takes_value(true)
                .value_name("FILE")
                .help("Save the downloaded tenhou.net/6 format log to FILE when --tenhou-id is specified. If FILE is \"-\", write to stdout"),
        )
        .arg(
            Arg::with_name("mjai-out")
                .long("mjai-out")
                .takes_value(true)
                .value_name("FILE")
                .help("Save the transformed mjai format log to FILE. If FILE is \"-\", write to stdout"),
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
                .help("Specify the directory of akochan. This will serves as the working directory of akochan process. Default value is the directory in which --akochan-exe is specified"),
        )
        .arg(
            Arg::with_name("akochan-exe")
                .short("e")
                .long("akochan-exe")
                .takes_value(true)
                .value_name("EXE")
                .help("Specify the executable file of akochan. Default value \"akochan/system.exe\""),
        )
        .arg(
            Arg::with_name("tactics-config")
                .short("c")
                .long("tactics-config")
                .takes_value(true)
                .value_name("FILE")
                .help("Specify the tactics config file for akochan. Default value \"tactics.json\""),
        )
        .get_matches();

    // get actor.
    // unwrap directly because it was already validated.
    let actor: u8 = matches.value_of("actor").unwrap().parse().unwrap();

    // load io specific options
    let in_file = matches.value_of_os("in-file");
    let out_file = matches.value_of_os("out-file");
    let tenhou_id = matches.value_of("tenhou-id");
    let tenhou_out = matches.value_of_os("tenhou-out");
    let mjai_out = matches.value_of_os("mjai-out");

    // get log reader, can be from a file, from stdin, or from HTTP stream
    let log_reader: Box<dyn Read> = {
        if let Some(tenhou_id_str) = tenhou_id {
            let log_stream = download_tenhou_log(tenhou_id_str)
                .with_context(|| format!("failed to download tenhou log ID={:?}", tenhou_id_str))?;

            // handle --tenhou-out
            if let Some(tenhou_str) = tenhou_out {
                if tenhou_str != "-" {
                    let tenhoufile = File::create(tenhou_str).with_context(|| {
                        format!("failed to create download out file {:?}", tenhou_str)
                    })?;
                    Box::new(TeeReader::new(log_stream, tenhoufile))
                } else {
                    Box::new(TeeReader::new(log_stream, io::stdout()))
                }
            } else {
                Box::new(log_stream)
            }
        } else {
            match in_file {
                Some(in_file_str) if in_file_str != "-" => {
                    Box::new(File::open(in_file_str).with_context(|| {
                        format!("failed to open tenhou log file {:?}", in_file_str)
                    })?)
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
    if let Some(mjai_out_str) = mjai_out {
        let mut w: Box<dyn Write> =
            if mjai_out_str != "-" {
                Box::from(File::create(mjai_out_str).with_context(|| {
                    format!("failed to create mjai out file {:?}", mjai_out_str)
                })?)
            } else {
                Box::from(io::stdout())
            };

        for event in &events {
            serde_json::to_writer(&mut w, event)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_out_str))?;
            writeln!(w)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_out_str))?;
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
    let tactics_config = {
        let path = matches
            .value_of_os("tactics-config")
            .map(PathBuf::from)
            .unwrap_or_else(|| "tactics.json".into());

        canonicalize(&path)
            .with_context(|| format!("failed to canonicalize tactics_config path {:?}", path))?
    };

    // load tactics_config for metadata
    let tactics: Tactics = {
        let f = File::open(&tactics_config)
            .with_context(|| format!("failed to open tactics_config {:?}", tactics_config))?;
        let j: TacticsJson = serde_json::from_reader(f)
            .with_context(|| format!("failed to parse tactics_config {:?}", tactics_config))?;
        j.tactics
    };

    // do the review
    let begin_review = chrono::Local::now();
    log!("start review, this may take serval minutes...");
    let reviews = review(akochan_exe, akochan_dir, tactics_config, &events, actor)
        .context("failed to review log")?;

    // determine whether the file can be opened after writing
    let opanable_file = match out_file {
        Some(out_file_str) => {
            if out_file_str != "-" {
                Some(out_file_str.to_owned())
            } else {
                None
            }
        }
        _ => {
            if let Some(id) = tenhou_id {
                Some(OsString::from(format!("{}&tw={}.html", id, actor)))
            } else {
                Some(OsString::from("report.html"))
            }
        }
    };

    // prepare output, can be a file or stdout
    let mut out: Box<dyn Write> = if let Some(out_file_str) = &opanable_file {
        Box::new(
            File::create(out_file_str)
                .with_context(|| format!("failed to create HTML report file {:?}", out_file_str))?,
        )
    } else {
        Box::new(io::stdout())
    };

    let now = chrono::Local::now();
    let parse_time = (begin_convert_log - begin_parse_log).to_std()?;
    let convert_time = (begin_review - begin_convert_log).to_std()?;
    let review_time = (now - begin_review).to_std()?;
    let meta = Metadata {
        jun_pt: &tactics.jun_pt,
        parse_time,
        convert_time,
        review_time,
        now,
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
        if let Some(out_file_str) = &opanable_file {
            opener::open(out_file_str).with_context(|| {
                format!(
                    "failed to open rendered HTML report file {:?}",
                    out_file_str
                )
            })?;
        }
    }

    log!("done");
    Ok(())
}
