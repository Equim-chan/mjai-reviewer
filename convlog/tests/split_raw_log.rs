mod testdata;

use convlog::*;
use serde_json;
use testdata::TESTDATA;

#[test]
fn test_split_by_kyoku() {
    TESTDATA.iter().for_each(|data| {
        let raw_log: tenhou::RawLog =
            serde_json::from_str(data).expect("failed to parse tenhou log");
        let splited = raw_log.split_by_kyoku();

        assert_eq!(splited.len(), raw_log.kyokus_count());
    });
}
