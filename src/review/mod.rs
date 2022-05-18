pub mod akochan;
pub mod mortal;

use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Review {
    Akochan(akochan::Review),
    Mortal(mortal::Review),
}
