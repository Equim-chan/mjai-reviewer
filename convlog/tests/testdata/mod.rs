pub struct TestCase {
    pub desc: &'static str,
    pub data: &'static str,
}

pub const TESTDATA: &[TestCase] = &[
    TestCase {
        desc: "chankan",
        data: include_str!("chankan.json"),
    },
    TestCase {
        desc: "complex_nakis_0",
        data: include_str!("complex_nakis_0.json"),
    },
    TestCase {
        desc: "complex_nakis_1",
        data: include_str!("complex_nakis_1.json"),
    },
    TestCase {
        desc: "confusing_nakis_0",
        data: include_str!("confusing_nakis_0.json"),
    },
    TestCase {
        desc: "confusing_nakis_1",
        data: include_str!("confusing_nakis_1.json"),
    },
    TestCase {
        desc: "confusing_nakis_2",
        data: include_str!("confusing_nakis_2.json"),
    },
    TestCase {
        desc: "confusing_nakis_3",
        data: include_str!("confusing_nakis_3.json"),
    },
    TestCase {
        desc: "confusing_nakis_4",
        data: include_str!("confusing_nakis_4.json"),
    },
    TestCase {
        desc: "confusing_nakis_5",
        data: include_str!("confusing_nakis_5.json"),
    },
    TestCase {
        desc: "confusing_nakis_6",
        data: include_str!("confusing_nakis_6.json"),
    },
    TestCase {
        desc: "confusing_nakis_7",
        data: include_str!("confusing_nakis_7.json"),
    },
    TestCase {
        desc: "double_kakan_then_chankan",
        data: include_str!("double_kakan_then_chankan.json"),
    },
    TestCase {
        desc: "four_reach",
        data: include_str!("four_reach.json"),
    },
    TestCase {
        desc: "kyushukyuhai",
        data: include_str!("kyushukyuhai.json"),
    },
    TestCase {
        desc: "double_ron",
        data: include_str!("double_ron.json"),
    },
    TestCase {
        desc: "ranked_game",
        data: include_str!("ranked_game.json"),
    },
    TestCase {
        desc: "rinshan",
        data: include_str!("rinshan.json"),
    },
    TestCase {
        desc: "ryukyoku",
        data: include_str!("ryukyoku.json"),
    },
    TestCase {
        desc: "suukantsu_0",
        data: include_str!("suukantsu_0.json"),
    },
    TestCase {
        desc: "suukantsu_1",
        data: include_str!("suukantsu_1.json"),
    },
];
