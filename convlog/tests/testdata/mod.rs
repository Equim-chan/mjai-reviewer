pub struct TestCase {
    pub description: &'static str,
    pub data: &'static str,
}

pub const TESTDATA: &[TestCase] = &[
    TestCase {
        description: "chankan",
        data: include_str!("chankan.json"),
    },
    TestCase {
        description: "complex_nakis_0",
        data: include_str!("complex_nakis_0.json"),
    },
    TestCase {
        description: "complex_nakis_1",
        data: include_str!("complex_nakis_1.json"),
    },
    TestCase {
        description: "confusing_nakis_0",
        data: include_str!("confusing_nakis_0.json"),
    },
    TestCase {
        description: "confusing_nakis_1",
        data: include_str!("confusing_nakis_1.json"),
    },
    TestCase {
        description: "confusing_nakis_2",
        data: include_str!("confusing_nakis_2.json"),
    },
    TestCase {
        description: "confusing_nakis_3",
        data: include_str!("confusing_nakis_3.json"),
    },
    TestCase {
        description: "confusing_nakis_4",
        data: include_str!("confusing_nakis_4.json"),
    },
    TestCase {
        description: "confusing_nakis_5",
        data: include_str!("confusing_nakis_5.json"),
    },
    TestCase {
        description: "confusing_nakis_6",
        data: include_str!("confusing_nakis_6.json"),
    },
    TestCase {
        description: "confusing_nakis_7",
        data: include_str!("confusing_nakis_7.json"),
    },
    TestCase {
        description: "double_kakan_then_chankan",
        data: include_str!("double_kakan_then_chankan.json"),
    },
    TestCase {
        description: "four_reach",
        data: include_str!("four_reach.json"),
    },
    TestCase {
        description: "kyushukyuhai",
        data: include_str!("kyushukyuhai.json"),
    },
    TestCase {
        description: "double_ron",
        data: include_str!("double_ron.json"),
    },
    TestCase {
        description: "ranked_game",
        data: include_str!("ranked_game.json"),
    },
    TestCase {
        description: "rinshan",
        data: include_str!("rinshan.json"),
    },
    TestCase {
        description: "ryukyoku",
        data: include_str!("ryukyoku.json"),
    },
    TestCase {
        description: "suukantsu_0",
        data: include_str!("suukantsu_0.json"),
    },
    TestCase {
        description: "suukantsu_1",
        data: include_str!("suukantsu_1.json"),
    },
];
