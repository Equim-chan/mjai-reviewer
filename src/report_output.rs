use std::ffi::OsString;

pub enum ReportOutput {
    File(OsString),
    Stdout,
}
