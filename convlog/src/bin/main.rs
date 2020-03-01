use convlog::{tenhou, tenhou_to_mjai};
use serde_json;
use std::io;

fn main() {
    let stdin = io::stdin();
    let tenhou_log = tenhou::Log::from_json_reader(stdin).expect("failed to parse tenhou log");
    tenhou_to_mjai(&tenhou_log)
        .expect("failed to transform tenhou log")
        .iter()
        .for_each(|event| println!("{}", serde_json::to_string(event).unwrap()));
}
