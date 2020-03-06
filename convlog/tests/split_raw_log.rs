mod testdata;

use convlog::*;
use serde_json;
use testdata::TESTDATA;

#[test]
fn test_split_by_kyoku() {
    TESTDATA.iter().for_each(|data| {
        let raw_log: tenhou::RawLog =
            serde_json::from_str(data).expect("failed to parse tenhou log");
        let splited_raw_logs = raw_log.split_by_kyoku();

        let log = tenhou::Log::from(raw_log.clone());
        let joined_kyokus: Vec<_> = splited_raw_logs
            .into_iter()
            .map(tenhou::RawLog::from)
            .map(tenhou::Log::from)
            .flat_map(|l| l.kyokus)
            .collect();
        let joined_logs = tenhou::Log {
            kyokus: joined_kyokus,
            ..log.clone()
        };

        let mjai_log = tenhou_to_mjai(&log).expect("failed to transform tenhou log");
        let mjai_log_joined = tenhou_to_mjai(&joined_logs).expect("failed to transform tenhou log");

        assert_eq!(mjai_log, mjai_log_joined);
    });
}
