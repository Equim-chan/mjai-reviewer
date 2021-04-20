mod testdata;

use convlog::*;
use testdata::{TestCase, TESTDATA};

#[test]
fn test_parse_and_convert() {
    TESTDATA.iter().for_each(|TestCase { description, data }| {
        let tenhou_log = tenhou::Log::from_json_str(data).expect(&*format!(
            "failed to parse tenhou log (case: {})",
            description
        ));
        let mjai_log = tenhou_to_mjai(&tenhou_log).expect(&*format!(
            "failed to transform tenhou log (case: {})",
            description
        ));

        assert!(mjai_log.len() >= 4);
    });
}
