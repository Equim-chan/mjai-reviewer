use crate::*;

use serde_json;

static TESTDATA: &[&str] = &[
    include_str!("testdata/chankan.json"),
    include_str!("testdata/four_reach.json"),
    include_str!("testdata/double_ron.json"),
    include_str!("testdata/ranked_game.json"),
    include_str!("testdata/rinshan.json"),
    include_str!("testdata/ryukyoku.json"),
    include_str!("testdata/suukantsu_0.json"),
    include_str!("testdata/suukantsu_1.json"),
];

#[test]
fn parse_and_convert() {
    TESTDATA.iter().for_each(|data| {
        let tenhou_log = tenhou::Log::from_json_string(data).expect("failed to parse tenhou log");
        let mjai_log = tenhou_to_mjai(&tenhou_log).expect("failed to transform tenhou log");

        assert!(!mjai_log.is_empty());
    });

    let raw = r#"{"title":["",""],"name":["","","",""],"rule":{"aka":1},"log":[[[4,0,0],[25000,25000,25000,25000],[47],[],[12,14,51,16,22,24,31,53,36,37,16,18,44],[42,"c171618",47,33,46,32,47,47,19,45,21,27,22],[44,31,60,60,60,60,60,60,60,60,60,42],[11,11,11,11,11,11,11,11,11,11,11,11,11],[11,11,11,29,43,11,26,11,11,11,18,38],[41,44,29,60,60,12,60,46,15,31,60,60],[11,11,11,11,11,11,11,11,11,11,11,11,11],[11,41,11,32,31,11,16,11,11,13,11,17],[41,60,11,60,60,42,60,37,45,60,"r35",60],[11,11,11,11,11,11,11,11,11,11,11,11,11],[11,11,11,11,27,11,11,11,11,11,11,11],[17,43,44,28,60,38,32,35,31,41,11,11],["不明"]]]}"#;
    let tenhou_log = tenhou::Log::from_json_string(raw).expect("failed to parse tenhou log");
    let mjai_log = tenhou_to_mjai(&tenhou_log).expect("failed to transform tenhou log");
    for log in &mjai_log {
        println!("{}", serde_json::to_string(log).unwrap());
    }
}
