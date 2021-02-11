use std::ffi::OsString;

pub enum LogSource {
    Tenhou(String),
    MahjongSoul(String),
    File(OsString),
    Stdin,
}

impl LogSource {
    pub fn default_output_filename(&self, actor: u8) -> OsString {
        match self {
            LogSource::Tenhou(id) => format!("{}&tw={}", id, actor).into(),
            LogSource::MahjongSoul(full_id) => {
                format!("{}_{}", mjsoul_log_id_from_full(full_id), actor).into()
            }
            LogSource::File(filename) => filename.clone(),
            LogSource::Stdin => "report".to_owned().into(),
        }
    }

    #[inline]
    pub fn log_id(&self) -> Option<&str> {
        match self {
            LogSource::Tenhou(id) => Some(&id),
            LogSource::MahjongSoul(full_id) => Some(mjsoul_log_id_from_full(full_id)),
            _ => None,
        }
    }
}

#[inline]
fn mjsoul_log_id_from_full(full_id: &str) -> &str {
    if let Some(underscore_idx) = full_id.find('_') {
        &full_id[..underscore_idx]
    } else {
        full_id
    }
}
