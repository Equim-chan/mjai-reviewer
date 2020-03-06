mod testdata;

use convlog::*;
use testdata::TESTDATA;

#[test]
fn test_parse_and_convert() {
    TESTDATA.iter().for_each(|data| {
        let tenhou_log = tenhou::Log::from_json_str(data).expect("failed to parse tenhou log");
        let mjai_log = tenhou_to_mjai(&tenhou_log).expect("failed to transform tenhou log");

        assert!(!mjai_log.is_empty());
    });
}
