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
            LogSource::Tenhou(id) => Some(id),
            LogSource::MahjongSoul(full_id) => Some(mjsoul_log_id_from_full(full_id)),
            _ => None,
        }
    }
}

#[inline]
fn mjsoul_log_id_from_full(full_id: &str) -> &str {
    full_id.find('_').map(|i| &full_id[..i]).unwrap_or(full_id)
}

fn deobfuse_mjsoul_log_id(id: &str) -> String {
    let mut ret = String::with_capacity(id.len());
    for (i, &code) in id.as_bytes().iter().enumerate() {
        let o = if (b'0'..=b'9').contains(&code) {
            code - b'0'
        } else if (b'a'..=b'z').contains(&code) {
            code - b'a' + 10
        } else {
            ret.push(code as char);
            continue;
        };

        let o = (o + 55 - i as u8) % 36;
        if o < 10 {
            ret.push((o + b'0') as char)
        } else {
            ret.push((o + b'a' - 10) as char)
        }
    }

    ret
}
