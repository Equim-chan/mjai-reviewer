#![deny(
    rust_2018_idioms,
    let_underscore_drop,
    clippy::assertions_on_result_states,
    clippy::bool_to_int_with_if,
    clippy::borrow_as_ptr,
    clippy::cloned_instead_of_copied,
    clippy::create_dir,
    clippy::debug_assert_with_mut_call,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_drop,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::equatable_if_let,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::format_push_string,
    clippy::from_iter_instead_of_collect,
    clippy::get_unwrap,
    clippy::implicit_clone,
    clippy::implicit_saturating_sub,
    clippy::imprecise_flops,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_unwrap_or,
    clippy::match_bool,
    clippy::match_same_arms,
    clippy::mut_mut,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::nonstandard_macro_braces,
    clippy::or_fun_call,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::redundant_else,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_if_nothing_returned,
    clippy::significant_drop_in_scrutinee,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_to_owned,
    clippy::trait_duplication_in_bounds,
    clippy::trivially_copy_pass_by_ref,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_join,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unseparated_literal_suffix,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq
)]

mod download;
mod log;
mod log_source;
mod opts;
mod raw_log_ext;
mod render;
mod review;
mod softmax;
mod state;
mod tactics;
mod tehai;

use crate::log_source::LogSource;
use crate::opts::{AkochanOptions, Engine, InputOptions, MortalOptions, Options, OutputOptions};
use crate::render::View;
use crate::review::{akochan, mortal, Review};
use chrono::SubsecRound;
use convlog::tenhou::{GameLength, Log, RawLog};
use convlog::tenhou_to_mjai;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};
use clap::{Parser, ValueEnum};
use serde_json as json;

macro_rules! canonicalize {
    ($path:ident) => {{
        let p = if $path.as_os_str().is_empty() {
            Path::new(".")
        } else {
            $path.as_ref()
        };
        dunce::canonicalize(p).with_context(|| {
            format!(
                "failed to canonicalize {}: \"{}\" (does it exist?)",
                stringify!($path),
                $path.display(),
            )
        })
    }};
}

enum ReportOutput {
    File(PathBuf),
    Stdout,
}

fn main() -> Result<()> {
    let Options {
        player_id,
        player_name,
        kyokus,
        no_review,
        verbose,
        engine,
        input_opts:
            InputOptions {
                in_file,
                tenhou_id,
                url,
            },
        output_opts:
            OutputOptions {
                out_file,
                tenhou_out,
                mjai_out,
                json,
                show_rating,
                without_log_viewer,
                anonymous,
                no_open,
                lang,
            },
        mortal_opts:
            MortalOptions {
                mortal_exe,
                mortal_cfg,
                temperature,
            },
        akochan_opts:
            AkochanOptions {
                akochan_dir,
                akochan_tactics,
                deviation_threshold,
            },
    } = Options::parse();

    // sometimes the log URL contains the actor info
    let mut player_id_opt = player_id;

    let log_source = if let Some(filename) = in_file {
        if filename == Path::new("-") {
            LogSource::Stdin
        } else {
            LogSource::File(filename)
        }
    } else if let Some(id) = tenhou_id {
        LogSource::Tenhou(id)
    } else if let Some(url) = url {
        let host = url.host_str().context("url does not have host")?;
        ensure!(
            host == "tenhou.net",
            "only logs from tenhou.net are supported",
        );

        let mut log = None;
        let mut tw = None;
        for (k, v) in url.query_pairs() {
            match &*k {
                "log" => log = Some(v.into_owned()),
                "tw" => {
                    let num: u8 = v.parse().context("\"tw\" must be a number")?;
                    if num >= 4 {
                        bail!("\"tw\" must be within 0-3, got {num}");
                    }
                    tw = Some(num);
                }
                _ => continue,
            };
            if log.is_some() && tw.is_some() {
                break;
            }
        }

        player_id_opt = player_id_opt.or(tw).or(Some(0));
        let id = log.with_context(|| format!("tenhou log ID not found in URL {url}"))?;
        LogSource::Tenhou(id)
    } else {
        LogSource::Stdin
    };

    let tenhou_out = tenhou_out
        .map(|filename| -> Result<(Box<dyn Write>, _)> {
            if filename == Path::new("-") {
                Ok((Box::new(io::stdout()), PathBuf::from("stdout")))
            } else {
                let file = File::create(&filename).with_context(|| {
                    format!("failed to create tenhou output file {}", filename.display())
                })?;
                Ok((Box::new(file), filename))
            }
        })
        .transpose()?;

    // download and parse tenhou.net/6 log
    let mut raw_log: RawLog = match &log_source {
        LogSource::Tenhou(id) => {
            let body = download::tenhou_log(id)
                .with_context(|| format!("failed to download tenhou log {id}"))?;
            if let Some((mut writer, filename)) = tenhou_out {
                writer.write_all(body.as_bytes()).with_context(|| {
                    format!(
                        "failed to write downloaded tenhou log to {}",
                        filename.display()
                    )
                })?;
            }
            json::from_str(&body).context("failed to parse tenhou.net/6 log")?
        }
        LogSource::File(filename) => {
            let mut file = File::open(filename)
                .with_context(|| format!("failed to open tenhou.net/6 log file {filename:?}"))?;
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

    if player_id_opt.is_none() {
        if let Some(player_name) = player_name {
            let names = raw_log.get_names();
            for (idx, n) in names.iter().enumerate() {
                if *n == player_name {
                    player_id_opt = Some(idx as u8);
                    break;
                }
            }
            ensure!(
                player_id_opt.is_some(),
                "there is no player named {player_name}, available players: {names:?}",
            );
        }
    }

    // apply filters
    if anonymous {
        raw_log.hide_names();
    }
    if let Some(expr) = kyokus {
        let filter = expr.parse().context("failed to parse kyoku filter")?;
        raw_log.filter_kyokus(&filter);
        ensure!(!raw_log.is_empty(), "no kyoku to review (invalid filter?)");
    }

    // clone the parsed raw log for possible reuse (split)
    let cloned_raw_log;
    let splitted_raw_logs = if without_log_viewer {
        None
    } else {
        cloned_raw_log = raw_log.clone();
        Some(cloned_raw_log.split_by_kyoku())
    };

    // convert from RawLog to Log.
    let log = Log::try_from(raw_log).context("invalid log")?;

    // convert from tenhou::Log to Vec<mjai::Event>
    let begin_convert_log = chrono::Local::now();
    log!("converting to mjai events...");
    let events =
        tenhou_to_mjai(&log).context("failed to convert tenhou.net/6 log into mjai format")?;

    if let Some(mjai_out) = mjai_out {
        let mut w: Box<dyn Write> = if mjai_out == Path::new("-") {
            Box::from(io::stdout())
        } else {
            let mjai_out_file = File::create(&mjai_out).with_context(|| {
                format!("failed to create mjai out file {:}", mjai_out.display())
            })?;
            Box::from(mjai_out_file)
        };

        for event in &events {
            let to_write = json::to_string(event).context("failed to serialize")?;
            writeln!(w, "{to_write}").with_context(|| {
                format!("failed to write to mjai out file {}", mjai_out.display())
            })?;
        }
    }

    if no_review {
        return Ok(());
    }
    // unwrap is safe because --engine is required when --no-review is not
    // present.
    let engine = engine.unwrap();

    if engine == Engine::Mortal && log.game_length != GameLength::Hanchan {
        bail!("Mortal supports hanchan games only");
    }

    // get player id
    let player_id = player_id_opt.context("a player ID is required for review")?;
    log!("players: {}", log.names.join(", "));
    log!("target: {} ({player_id})", log.names[player_id as usize]);

    let begin_review = chrono::Local::now();
    let review = match engine {
        Engine::Mortal => {
            let mortal_exe = canonicalize!(mortal_exe)?;
            let mortal_cfg = canonicalize!(mortal_cfg)?;
            let reviewer = mortal::Reviewer {
                mortal_exe: &mortal_exe,
                mortal_cfg: &mortal_cfg,
                events: &events,
                player_id,
                temperature,
                verbose,
            };
            let result = reviewer.review().context("failed to review")?;
            Review::Mortal(result)
        }
        Engine::Akochan => {
            let akochan_exe: PathBuf = [&akochan_dir, Path::new("system.exe")]
                .into_iter()
                .collect();
            let akochan_dir = canonicalize!(akochan_dir)?;
            let akochan_exe = canonicalize!(akochan_exe)?;
            let akochan_tactics = canonicalize!(akochan_tactics)?;
            let reviewer = akochan::Reviewer {
                akochan_exe: &akochan_exe,
                akochan_dir: &akochan_dir,
                tactics_config: &akochan_tactics,
                events: &events,
                player_id,
                deviation_threshold,
                verbose,
            };
            let result = reviewer.review().context("failed to review")?;
            Review::Akochan(result)
        }
    };

    // determine output file
    let out = if let Some(filename) = out_file {
        if filename == Path::new("-") {
            ReportOutput::Stdout
        } else {
            ReportOutput::File(filename)
        }
    } else {
        let suffix = if json { ".json" } else { ".html" };
        let mut filename = log_source
            .default_output_filename(engine, player_id)
            .into_os_string();
        filename.push(suffix);
        ReportOutput::File(PathBuf::from(filename))
    };

    // prepare output, can be a file or stdout
    let mut out_write: Box<dyn Write> = match &out {
        ReportOutput::File(filename) => {
            let file = File::create(filename).with_context(|| {
                format!("failed to create output report file {}", filename.display())
            })?;
            Box::new(file)
        }
        ReportOutput::Stdout => Box::new(io::stdout()),
    };

    let now = chrono::Local::now();
    let loading_time =
        (begin_review.trunc_subsecs(3) - begin_convert_log.trunc_subsecs(3)).to_std()?;
    let review_time = (now.trunc_subsecs(3) - begin_review.trunc_subsecs(3)).to_std()?;
    let lang_value = lang.to_possible_value().unwrap();

    // render the HTML report page or JSON
    let view = View {
        engine,
        game_length: log.game_length,
        log_id: if anonymous { None } else { log_source.log_id() },
        loading_time,
        review_time,
        show_rating,
        version: env!("CARGO_PKG_VERSION"),

        review,
        player_id,

        split_logs: splitted_raw_logs.as_deref(),
        lang: lang_value.get_name(),
    };
    log!("writing output...");
    if json {
        json::to_writer(&mut out_write, &view).context("failed to write JSON result")?;
    } else {
        view.render(&mut out_write)
            .context("failed to render HTML report")?;
    }
    out_write.flush().context("failed to flush output")?;
    log!("complete");

    // open the output page
    if !json && !no_open {
        if let ReportOutput::File(filepath) = out {
            opener::open(&filepath).with_context(|| {
                format!(
                    "failed to open rendered HTML report file {}",
                    filepath.display(),
                )
            })?;
        }
    }

    Ok(())
}
