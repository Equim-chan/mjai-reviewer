mod download;
mod log;
mod render;
mod review;

use anyhow::{Context, Result};
use clap::{App, Arg};
use convlog::tenhou;
use download::download_tenhou_log;
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
            Arg::with_name("download-out")
                .long("download-out")
                .takes_value(true)
                .value_name("FILE")
                .help("Save the downloaded tenhou.net/6 format log to FILE when --tenhou-id is specified"),
        )
        .arg(
            Arg::with_name("mjai-out")
                .long("mjai-out")
                .takes_value(true)
                .value_name("FILE")
                .help("Save the transformed mjai format log to FILE"),
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

        path.canonicalize().context(format!(
            "failed to canonicalize akochan_exe path {:?}",
            path
        ))?
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

        path.canonicalize().context(format!(
            "failed to canonicalize akochan_dir path {:?}",
            path
        ))?
    };
    let tactics_config = {
        let path = matches
            .value_of_os("tactics-config")
            .map(PathBuf::from)
            .unwrap_or_else(|| "tactics.json".into());

        path.canonicalize().context(format!(
            "failed to canonicalize tactics_config path {:?}",
            path
        ))?
    };

    // load other options
    let in_file = matches.value_of_os("in-file");
    let out_file = matches.value_of_os("out-file");
    let tenhou_id = matches.value_of("tenhou-id");
    let download_out = matches.value_of_os("download-out");
    let mjai_out = matches.value_of_os("mjai-out");

    // get log reader, can be from a file, from stdin, or from HTTP stream
    let log_reader: Box<dyn Read> = {
        if let Some(tenhou_id_str) = tenhou_id {
            let log_stream = download_tenhou_log(tenhou_id_str)
                .with_context(|| format!("failed to download tenhou log ID={:?}", tenhou_id_str))?;

            if let Some(download_out_str) = download_out {
                let download_out_file = File::create(download_out_str).with_context(|| {
                    format!("failed to create download out file {:?}", download_out_str)
                })?;
                Box::new(TeeReader::new(log_stream, download_out_file))
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
    log!("converting to mjai events...");
    let events =
        convlog::tenhou_to_mjai(&log).context("failed to convert tenhou log into mjai format")?;
    if let Some(mjai_out_str) = mjai_out {
        let mut mjai_file = File::create(mjai_out_str)
            .with_context(|| format!("failed to create mjai out file {:?}", mjai_out_str))?;
        for event in &events {
            serde_json::to_writer(&mut mjai_file, event)
                .with_context(|| format!("failed to write to mjai out file {:?}", mjai_file))?;
        }
    }

    // do the review
    log!("start review...");
    let reviews = review(
        akochan_exe,
        akochan_dir,
        tactics_config,
        &events,
        actor,
    )
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

    // render the HTML report page
    log!("rendering output...");
    if let Some(l) = cloned_raw_log {
        render(&mut out, &reviews, actor, Some(&l.split_by_kyoku()))
    } else {
        render(&mut out, &reviews, actor, None)
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
