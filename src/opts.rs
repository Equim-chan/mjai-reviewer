use std::ffi::OsStr;
use std::fmt;
use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, ValueEnum};
use serde::Serialize;
use url::Url;

const ABOUT: &str = r#"🔍🀄️ Review your Tenhou or Mahjong Soul (Jantama) log with mjai-compatible mahjong AIs.

Basic usage:
  $ mjai-reviewer -e mortal -u "https://tenhou.net/0/?log=2019050417gm-0029-0000-4f2a8622&tw=2"

For more details, please visit the repo at <https://github.com/Equim-chan/mjai-reviewer>."#;

#[derive(Debug, Parser)]
#[clap(version, about = ABOUT)]
#[clap(group(
    ArgGroup::new("input-methods")
        .args(&["in_file", "tenhou_id", "url"]),
))]
pub struct Options {
    /// The ID of the player to review, which is a number within 0-3. It is the
    /// number after "&tw=" in Tenhou's log URL, namely, the player sitting at
    /// the East at E1 is 0, and his shimocha (right) will be 1, toimen (across)
    /// will be 2, kamicha (left) will be 3. This option has higher priority
    /// over the "&tw=" in --url if specified.
    #[clap(short = 'a', long, value_name = "ID", value_parser = parse_player_id)]
    pub player_id: Option<u8>,

    /// The display name of the player to review. This option has higher
    /// priority over the "&tw=" in --url if specified.
    #[clap(short = 'n', long, value_name = "NAME", conflicts_with = "player_id")]
    pub player_name: Option<String>,

    #[clap(flatten, next_help_heading = "Input Options")]
    pub input_opts: InputOptions,

    #[clap(flatten, next_help_heading = "Output Options")]
    pub output_opts: OutputOptions,

    /// Kyokus to review. If LIST is empty, review all kyokus. Example:
    /// "E1,E4,S3.1", which means to review East-1, East-4, and South-3-1.
    #[clap(short, long, value_name = "LIST")]
    pub kyokus: Option<String>,

    /// Do not review at all, but only download and save files.
    #[clap(long)]
    pub no_review: bool,

    /// Print verbose logs.
    #[clap(short, long)]
    pub verbose: bool,

    /// The engine to use for review.
    #[clap(
        short,
        long,
        value_enum,
        required_unless_present = "no_review",
        requires = "input-methods"
    )]
    pub engine: Option<Engine>,

    #[clap(flatten, next_help_heading = "Mortal Options")]
    pub mortal_opts: MortalOptions,

    #[clap(flatten, next_help_heading = "Akochan Options")]
    pub akochan_opts: AkochanOptions,
}

#[derive(Debug, Args)]
pub struct InputOptions {
    /// The name of a tenhou.net/6 format log file to input. If FILE is "-" or
    /// empty, read from stdin.
    #[clap(short, long, value_name = "FILE")]
    pub in_file: Option<PathBuf>,

    /// The ID of a Tenhou log to review. Example:
    /// "2019050417gm-0029-0000-4f2a8622".
    #[clap(short, long, value_name = "ID")]
    pub tenhou_id: Option<String>,

    /// Tenhou log URL, as an alternative to --tenhou-id.
    #[clap(short, long, value_name = "URL", value_parser)]
    pub url: Option<Url>,
}

#[derive(Debug, Args)]
pub struct OutputOptions {
    /// The name of the generated HTML/JSON report file to output. If FILE is
    /// "-", write to stdout; if FILE is empty, write to
    /// "{engine}-{tenhou_id}&tw={actor}.{format}" if a Tenhou log ID is known,
    /// "{engine}-report.{format}" otherwise.
    #[clap(short, long, value_name = "FILE")]
    pub out_file: Option<PathBuf>,

    /// Save the downloaded tenhou.net/6 format log to FILE, which requires
    /// --tenhou-id to be specified. If FILE is "-", write to stdout.
    #[clap(long, value_name = "FILE")]
    pub tenhou_out: Option<PathBuf>,

    /// Save the converted mjai format log to FILE. If FILE is "-", write to
    /// stdout.
    #[clap(long, value_name = "FILE")]
    pub mjai_out: Option<PathBuf>,

    /// Output review result in JSON instead of HTML.
    #[clap(long)]
    pub json: bool,

    /// Do not include log viewer in the generated HTML report.
    #[clap(long)]
    pub without_log_viewer: bool,

    /// Do not include player names in the generated HTML report.
    #[clap(long)]
    pub anonymous: bool,

    /// Do not automatically open the output file in browser.
    #[clap(long)]
    pub no_open: bool,
}

#[derive(Debug, Args)]
pub struct MortalOptions {
    #[clap(
        long,
        value_name = "FILE",
        default_value_os = OsStr::new("./mortal/mortal")
    )]
    pub mortal_exe: PathBuf,

    #[clap(
        long,
        value_name = "FILE",
        default_value_os = OsStr::new("./mortal/config.toml")
    )]
    pub mortal_cfg: PathBuf,

    #[clap(long, value_name = "TEMP", default_value = "0.1", value_parser = parse_temperature)]
    pub temperature: f32,
}

#[derive(Debug, Args)]
pub struct AkochanOptions {
    #[clap(
        long,
        value_name = "DIR",
        default_value_os = OsStr::new("./akochan")
    )]
    pub akochan_dir: PathBuf,

    #[clap(
        long,
        value_name = "FILE",
        default_value_os = OsStr::new("./akochan/tactics.json")
    )]
    pub akochan_tactics: PathBuf,

    #[clap(long, value_name = "DEV", default_value = "0.05")]
    pub deviation_threshold: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ValueEnum)]
pub enum Engine {
    Mortal,
    Akochan,
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Mortal => "mortal",
            Self::Akochan => "akochan",
        };
        fmt::Display::fmt(s, f)
    }
}

fn parse_player_id(s: &str) -> Result<u8, String> {
    let id = s.parse::<u8>().map_err(|e| e.to_string())?;
    if id >= 4 {
        Err(format!("{s} is not within 0-3"))
    } else {
        Ok(id)
    }
}

fn parse_temperature(s: &str) -> Result<f32, String> {
    let v = s.parse::<f32>().map_err(|e| e.to_string())?;
    if v <= 0. {
        Err(format!("{s} is not greater than zero"))
    } else {
        Ok(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn cli_parse() {
        Options::command().debug_assert();
    }
}
