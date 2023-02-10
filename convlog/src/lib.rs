//! Provides methods to transform mahjong logs from tenhou.net/6 format into
//! mjai format.

#![deny(
    rust_2018_idioms,
    let_underscore_drop,
    clippy::uninlined_format_args,
    clippy::unseparated_literal_suffix,
    clippy::must_use_candidate,
    clippy::redundant_else,
    clippy::manual_assert,
    clippy::manual_ok_or,
    clippy::needless_for_each,
    clippy::needless_continue,
    clippy::map_unwrap_or,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::get_unwrap,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::inefficient_to_string,
    clippy::let_unit_value,
    clippy::cloned_instead_of_copied,
    clippy::debug_assert_with_mut_call,
    clippy::equatable_if_let,
    clippy::default_union_representation,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::lossy_float_literal,
    clippy::implicit_clone,
    clippy::implicit_saturating_sub,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_if_nothing_returned,
    clippy::str_to_string,
    clippy::string_to_string,
    clippy::string_lit_as_bytes,
    clippy::trivially_copy_pass_by_ref,
    clippy::unicode_not_nfc,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unused_async,
    clippy::useless_let_if_seq,
    clippy::mut_mut,
    clippy::nonstandard_macro_braces,
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr
)]

mod conv;
mod kyoku_filter;
mod macros;
mod mjai;
mod tile;

pub mod tenhou;

pub use conv::tenhou_to_mjai;
pub use conv::ConvertError;
pub use kyoku_filter::KyokuFilter;
pub use mjai::Event;
pub use tile::{tile_set_eq, Tile};
