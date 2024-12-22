//! `bevy_undo_redo` is an implementation of an undo/redo system for the Bevy game engine.

// NOTE: Lints are sorted using the following process. For each lint you want to find or add to the
// list, follow the steps here in order:
// 1. Where a lint does not have a `reason = ...` specified, it must go to the bottom of the lint
//    attribute list, grouped into the appropriate lint attribute at the bottom.
// 2. Where two or more lints share a single lint attribute, lints must be placed in alphabetical
//    order, ignoring the `clippy::` prefix for clippy lints.
// 3. When sorting the list of lint attributes, it must be sorted in alphabetical order, ignoring
//    the `clippy::` prefix for clippy lints. The top-most lint in the lint attribute is what
//    determines where the lint attribute is sorted.
//
// When using `#[expect]` to declare an exception to one of the lints here, sort lints
// alphabetically, ignoring the `clippy::` prefix for clippy lints.
#![forbid(
	clippy::allow_attributes_without_reason,
	reason = "All exceptions to any lints must justify their reasoning."
)]
#![deny(
	// `clippy::missing_panics_docs` is considered a forbidden lint. However, it has some false
	// positives which trigger the lint unnecessarily. For an example, see `History::push()`'s code.
	//
	// All exceptions for `clippy::missing_panics_docs` must be marked as `#[expect()]`, and a reason
	// must be given.
	clippy::missing_panics_doc,
)]
#![forbid(
	clippy::allow_attributes,
	clippy::cargo_common_metadata,
	dead_code,
	clippy::doc_markdown,
	clippy::empty_docs,
	//missing_docs,
	//clippy::missing_docs_in_private_items,
	clippy::missing_enforced_import_renames,
	clippy::missing_errors_doc,
	// `clippy::missing_panics_docs` is set to deny - see its lint attribute for why.
	clippy::missing_safety_doc,
	clippy::module_name_repetitions,
	clippy::multiple_crate_versions,
	clippy::must_use_candidate,
	clippy::semicolon_if_nothing_returned,
	clippy::semicolon_inside_block,
	clippy::std_instead_of_core,
)]

pub(crate) mod error;
pub mod history;
pub mod operation;
pub(crate) mod undoredo;

pub use crate::{error::*, history::*, operation::Operation, undoredo::*};
