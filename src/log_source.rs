use std::ffi::OsString;

pub enum LogSource {
    Tenhou(String),
    MahjongSoul(String),
    File(OsString),
    Stdin,
}

impl LogSource {
    pub fn mjsoul_full_id_with_deobfuse(raw: &str) -> Self {
        let seps: Vec<_> = raw.split('_').collect();
        if seps.len() == 3 && seps[2] == "2" {
            let real_id = deobfuse_mjsoul_log_id(seps[0]);
            LogSource::MahjongSoul(format!("{}_{}", real_id, seps[1]))
        } else {
            LogSource::MahjongSoul(raw.to_owned())
        }
    }

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

fn deobfuse_mjsoul_log_id(id: &str) -> String {
    const ZERO: u8 = b'0';
    const ALPHA: u8 = b'a';

    let mut ret = String::with_capacity(id.len());
    for (i, &code) in id.as_bytes().iter().enumerate() {
        let o = if (ZERO..ZERO + 10).contains(&code) {
            code - ZERO
        } else if (ALPHA..ALPHA + 26).contains(&code) {
            code - ALPHA + 10
        } else {
            ret.push(code as char);
            continue;
        };

        let o = (o + 55 - i as u8) % 36;
        if o < 10 {
            ret.push((o + ZERO) as char)
        } else {
            ret.push((o + ALPHA - 10) as char)
        }
    }

    ret
}
