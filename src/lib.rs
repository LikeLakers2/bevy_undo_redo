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
#![forbid(
	clippy::missing_enforced_import_renames,
	reason = "We require vague or often-renamed imports to be given a more meaningful name."
)]
#![forbid(
	//missing_docs,
	//clippy::missing_docs_in_private_items,
	clippy::missing_errors_doc,
	// `clippy::missing_panics_docs` is set to deny - see its lint attribute for why.
	clippy::missing_safety_doc,
	reason = "Extensively documenting all items is a good idea, as it reduces the workload when needing to (re)learn the codebase."
)]
#![deny(
	// `clippy::missing_panics_docs` is considered a forbidden lint. However, it has some false
	// positives which trigger the lint unnecessarily. For an example, see `History::push()`'s code.
	//
	// All exceptions for `clippy::missing_panics_docs` must be marked as `#[expect()]`, and a reason
	// must be given.
	clippy::missing_panics_doc,
	reason = "Extensively documenting all items is a good idea, as it reduces the workload when needing to (re)learn the codebase."
)]
#![forbid(
	clippy::allow_attributes,
	clippy::cargo_common_metadata,
	//dead_code
	clippy::doc_markdown,
	clippy::module_name_repetitions,
	clippy::multiple_crate_versions,
	clippy::must_use_candidate,
	clippy::semicolon_if_nothing_returned,
	clippy::semicolon_inside_block,
	clippy::std_instead_of_core,
)]

use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	num::NonZeroUsize,
};
use std::collections::VecDeque;

use bevy_ecs::system::{Commands, Resource};

use crate::operation::Operation;

pub mod operation;

#[derive(Default, Resource)]
pub struct UndoRedo {
	history: History<Box<dyn Operation>>,
	/// A list of operations that have been pushed to this [`UndoRedo`], but have not been applied
	/// to the World.
	queued_operations: VecDeque<Box<dyn Operation>>,
}

impl UndoRedo {
	pub fn push_operation<O: Operation>(&mut self, operation: O) {
		self.queued_operations.push_back(Box::new(operation));
	}

	/// Applies all operations that are queued. Any actions that have been undone, but not
	/// subsequently redone, will be lost.
	///
	/// # Errors
	/// * [`Error::NoWorkAvailable`] - There are no queued operations available to apply.
	pub fn apply_queued_operations(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no queued operations available, we have no work to do. Let the caller know.
		if self.queued_operations.is_empty() {
			return Err(Error::NoWorkAvailable);
		}

		let queued_operations = self.queued_operations.drain(..);

		for operation in queued_operations {
			operation.apply(commands);
			self.history.push(operation);
		}

		self.history.undone.clear();
		Ok(())
	}

	/// Applies the last undone operation, if any.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - No operations have been undone since the last time (if any)
	///   queued operations were applied.
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		let item = self.history.redo()?;

		// Submit all the commands needed to apply...
		item.apply(commands);

		Ok(())
	}

	/// Undoes the last committed operation, if any.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - There are no operations available to undo.
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		let item = self.history.undo()?;

		// Submit all the commands needed to undo...
		item.undo(commands);

		Ok(())
	}
}

pub struct History<T> {
	/// A list of all operations that have been committed, in the order they were committed.
	committed: VecDeque<T>,
	/// A list of all operations that were committed, but have subsequently been undone. Flushing
	/// the queued operations list will clear this list.
	// TODO: Document on the functions that flushing the queued operations list will clear this list
	// "Any operations that have been undone and not subsequently redone will be lost to time"
	//
	// TODO: Maybe combine this into `history` and use a `usize` as a cursor for which ops are
	// committed and which were undone?
	undone: Vec<T>,
	/// The maximum length of this history. Any committed operations past this limit will be
	/// automatically culled.
	limit: Option<NonZeroUsize>,
}

impl<T> History<T> {
	pub fn push(&mut self, item: T) {
		if let Some(limit) = self.limit {
			// If we have more committed operations than the limit, remove items from the committed
			// list until we're at the limit. Then, remove one more to make space for the operation
			// we're about to push.
			while limit.get() <= self.committed.len() {
				self.committed.pop_front();
			}
		}

		self.committed.push_back(item);
	}

	/// [TODO: Description]
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - No operations have been undone since the last time (if any)
	///   queued operations were applied.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause those panics are handled beforehand."
	)]
	pub fn redo(&mut self) -> Result<&T, Error> {
		// If there are no operations in the history, we have no work to do. Let the caller know.
		if self.undone.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an operation off the end of the undone list...
		//
		// NOTE: This cannot panic, as we just verified that `!self.undone.is_empty()`.
		let last_undone_operation = self.undone.pop().expect("undone list should not be empty");

		// And add that item to the end of the committed list.
		self.committed.push_back(last_undone_operation);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_undone_operation`, as Rust seems to yell
		// at us if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.committed`.
		let item_ref = self
			.committed
			.back()
			.expect("committed list should not be empty");

		Ok(item_ref)
	}

	/// [TODO: Description]
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - There are no operations available to undo.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause those panics are handled beforehand."
	)]
	pub fn undo(&mut self) -> Result<&T, Error> {
		// If there are no operations in the history, we have no work to do. Let the caller know.
		if self.committed.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an operation off the end of the history...
		//
		// NOTE: This cannot panic, as we just verified that `!self.committed.is_empty()`.
		let last_committed_operation = self
			.committed
			.pop_back()
			.expect("committed list should not be empty");

		// And add that item to the end of the undone list.
		self.undone.push(last_committed_operation);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_committed_operation`, as Rust seems to
		// yell at us if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.undone`.
		let item_ref = self.undone.last().expect("undone list should not be empty");

		Ok(item_ref)
	}
}

// Manually impl Default, to avoid putting a bound on T.
impl<T> Default for History<T> {
	fn default() -> Self {
		Self {
			committed: VecDeque::new(),
			undone: Vec::new(),
			limit: None,
		}
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	NoApplicableHistory,
	NoWorkAvailable,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let msg = match self {
			Self::NoApplicableHistory => {
				"No applicable history available to perform this operation"
			}
			Self::NoWorkAvailable => "No operation available to apply",
		};

		write!(f, "{msg}")
	}
}
