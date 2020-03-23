use std::io;

use convlog;
use convlog::tenhou;
use serde_json as json;

fn main() {
    let stdin = io::stdin();

    let tenhou_log_raw: tenhou::RawLog =
        json::from_reader(stdin).expect("failed to parse tenhou log");
    let tenhou_log = tenhou::Log::from(tenhou_log_raw);

    convlog::tenhou_to_mjai(&tenhou_log)
        .expect("failed to transform tenhou log")
        .iter()
        .for_each(|event| println!("{}", json::to_string(event).unwrap()));
}
