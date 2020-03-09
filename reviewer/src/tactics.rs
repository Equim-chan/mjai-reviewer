use serde::Deserialize;

#[derive(Deserialize)]
pub struct TacticsJson {
    pub tactics: Tactics,
}

#[derive(Deserialize)]
pub struct Tactics {
    pub jun_pt: [i32; 4],
}
