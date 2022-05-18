use crate::opts::Engine;
use std::path::PathBuf;

pub enum LogSource {
    Tenhou(String),
    File(PathBuf),
    Stdin,
}

impl LogSource {
    pub fn default_output_filename(&self, engine: Engine, player_id: u8) -> PathBuf {
        let engine_str = engine.to_string().to_lowercase();
        match self {
            LogSource::Tenhou(id) => format!("{engine_str}-{id}&tw={player_id}").into(),
            LogSource::File(filename) => filename.clone(),
            LogSource::Stdin => format!("{engine_str}_report").into(),
        }
    }

    #[inline]
    pub fn log_id(&self) -> Option<&str> {
        match self {
            LogSource::Tenhou(id) => Some(id),
            _ => None,
        }
    }
}
