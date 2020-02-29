use convlog::*;

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
}
