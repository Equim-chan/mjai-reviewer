//! Provides methods to transform mahjong logs from tenhou.net/6 format into
//! mjai format.

#![allow(clippy::manual_range_patterns)] // because of matches_tu8
#![deny(
    rust_2018_idioms,
    let_underscore_drop,
    clippy::assertions_on_result_states,
    clippy::bool_to_int_with_if,
    clippy::borrow_as_ptr,
    clippy::cloned_instead_of_copied,
    clippy::create_dir,
    clippy::debug_assert_with_mut_call,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_drop,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::equatable_if_let,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::format_push_string,
    clippy::from_iter_instead_of_collect,
    clippy::get_unwrap,
    clippy::implicit_clone,
    clippy::implicit_saturating_sub,
    clippy::imprecise_flops,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_unwrap_or,
    clippy::match_bool,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::mut_mut,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::nonstandard_macro_braces,
    clippy::or_fun_call,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::redundant_else,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_if_nothing_returned,
    clippy::significant_drop_in_scrutinee,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_to_owned,
    clippy::trait_duplication_in_bounds,
    clippy::trivially_copy_pass_by_ref,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_join,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unseparated_literal_suffix,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq
)]

mod conv;
mod kyoku_filter;
mod macros;
mod mjai;
mod tile;

pub mod tenhou;

pub use conv::ConvertError;
pub use conv::tenhou_to_mjai;
pub use kyoku_filter::KyokuFilter;
pub use mjai::Event;
pub use tile::{Tile, tile_set_eq};
