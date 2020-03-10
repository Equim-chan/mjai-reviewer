use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct TacticsJson {
    pub tactics: Tactics,
}

#[derive(Serialize, Deserialize)]
pub struct Tactics {
    pub jun_pt: [i32; 4],

    #[serde(flatten)]
    pub other_fields: HashMap<String, Value>,
}
