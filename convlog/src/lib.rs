//! Provides methods to transform mahjong logs from tenhou.net/6 format into
//! mjai format.

mod conv;
pub mod mjai;
pub mod pai;
pub mod tenhou;

pub use conv::tenhou_to_mjai;
pub use conv::ConvertError;
pub use pai::Pai;
