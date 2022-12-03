mod testdata;

use convlog::tenhou::{Log, RawLog};
use convlog::tenhou_to_mjai;
use testdata::{TestCase, TESTDATA};

use serde_json as json;

#[test]
fn test_split_by_kyoku() {
    TESTDATA.iter().for_each(|TestCase { desc, data }| {
        let raw_log: RawLog = json::from_str(data)
            .unwrap_or_else(|_| panic!("failed to parse tenhou log (case: {desc})"));
        let splited_raw_logs = raw_log.split_by_kyoku();

        let log =
            Log::try_from(raw_log.clone()).unwrap_or_else(|_| panic!("invalid log (case: {desc})"));
        let joined_kyokus: Vec<_> = splited_raw_logs
            .into_iter()
            .map(RawLog::from)
            .map(|l| Log::try_from(l).unwrap_or_else(|_| panic!("invalid log (case: {desc})")))
            .flat_map(|l| l.kyokus)
            .collect();
        let joined_logs = Log {
            kyokus: joined_kyokus,
            ..log.clone()
        };

        let mjai_log = tenhou_to_mjai(&log)
            .unwrap_or_else(|_| panic!("failed to transform tenhou (case: {desc})"));
        let mjai_log_joined = tenhou_to_mjai(&joined_logs)
            .unwrap_or_else(|_| panic!("failed to transform tenhou (case: {desc})"));

        assert_eq!(mjai_log, mjai_log_joined);
    });
}
