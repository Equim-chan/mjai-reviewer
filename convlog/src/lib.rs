//! Provides methods to transform mahjong logs from tenhou.net/6 format into
//! mjai format.

mod conv;
pub mod mjai;
mod pai;
pub mod tenhou;

pub use conv::{tenhou_to_mjai, ConvertError};
pub use pai::Pai;

#[cfg(test)]
mod tests;
