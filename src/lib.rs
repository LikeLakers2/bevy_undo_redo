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
	missing_docs,
	clippy::missing_docs_in_private_items,
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

use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	num::NonZeroUsize,
};
use std::collections::VecDeque;

use bevy_ecs::system::{Commands, Resource};

use crate::operation::Operation;

pub mod operation;

/// A high-level interface for implementing undo/redo operations.
#[derive(Default, Resource)]
pub struct UndoRedo {
	/// The collection which manages the list of committed operations, and acts as a pointer into
	/// that set of items.
	history: History<Box<dyn Operation>>,
	/// A list of operations that have been pushed to this [`UndoRedo`], but have not been applied
	/// to the World.
	queued_operations: VecDeque<Box<dyn Operation>>,
}

impl UndoRedo {
	/// Pushes an operation into the list of queued operations.
	///
	/// After pushing one or more operations, call [`Self::apply_queued_operations()`] to apply the
	/// operation(s) to the [`World`].
	///
	/// [`World`]: bevy_ecs::world::World
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

/// A collection which holds a set of items that represents the history of something, and acts as a
/// cursor into that set of items.
///
/// Unlike [`UndoRedo`], this struct does not affect a [`World`] when items are pushed to it. It
/// only acts as a pointer into a set of items.
///
/// [`World`]: bevy_ecs::world::World
pub struct History<T> {
	/// A list of all items that have been committed, in the order they were committed. The
	/// front-most item is the oldest committed item, and the back-most item is the newest committed
	/// item.
	committed: VecDeque<T>,
	/// A list of all items that were committed, but have subsequently been undone. Items at the end
	/// of the list are the most recently undone.
	// NOTE: Because we only care about items at one end of this list, we use a Vec rather than a
	// VecDeque, to gain a small amount of free performance.
	undone: Vec<T>,
	/// The maximum length of this history. Any committed items past this limit will be
	/// automatically culled the next time an item is pushed.
	pub limit: Option<NonZeroUsize>,
}

impl<T> History<T> {
	/// Pushes an item to the history.
	///
	/// If a history limit is set, any items past the limit will be removed, plus one more to make
	/// space for the item being pushed.
	pub fn push(&mut self, item: T) {
		if let Some(limit) = self.limit {
			// If we have more committed items than the limit, remove items from the committed
			// list until we're at the limit. Then, remove one more to make space for the item we're
			// about to push.
			while limit.get() <= self.committed.len() {
				self.committed.pop_front();
			}
		}

		self.committed.push_back(item);
	}

	/// Marks the last undone item as "committed", and returns a reference to it.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - If there is no history available to redo. This usually
	///   occurs if there haven't been any calls to [`Self::undo()`] since the last time an item was
	///   pushed.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause those panics are handled beforehand."
	)]
	pub fn redo(&mut self) -> Result<&T, Error> {
		// If there are no items in the history, we have no work to do. Let the caller know.
		if self.undone.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an item off the end of the undone list...
		//
		// NOTE: This cannot panic, as we just verified that `!self.undone.is_empty()`.
		let last_undone_item = self.undone.pop().expect("undone list should not be empty");

		// And add that item to the end of the committed list.
		self.committed.push_back(last_undone_item);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_undone_item`, as Rust seems to yell at us
		// if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.committed`.
		let item_ref = self
			.committed
			.back()
			.expect("committed list should not be empty");

		Ok(item_ref)
	}

	/// Marks the last committed item as "undone", and returns a reference to it.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - If there is no history available to undo.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause those panics are handled beforehand."
	)]
	pub fn undo(&mut self) -> Result<&T, Error> {
		// If there are no items in the history, we have no work to do. Let the caller know.
		if self.committed.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an item off the end of the history...
		//
		// NOTE: This cannot panic, as we just verified that `!self.committed.is_empty()`.
		let last_committed_item = self
			.committed
			.pop_back()
			.expect("committed list should not be empty");

		// And add that item to the end of the undone list.
		self.undone.push(last_committed_item);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_committed_item`, as Rust seems to yell at
		// us if we try.
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

/// The error type for history-type operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	/// There is no applicable history is available for this operation.
	NoApplicableHistory,
	/// There is no operation available to apply.
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
