use std::{
	collections::VecDeque,
	fmt::{Display, Formatter, Result as FmtResult},
	num::NonZeroUsize,
};

use bevy_ecs::system::{Commands, Resource};

use crate::operation::Operation;

pub mod operation;

#[derive(Default, Resource)]
pub struct UndoRedo {
	history: History<Box<dyn Operation>>,
	/// A list of operations that have been pushed to this UndoRedo, but have not been applied to
	/// the World.
	queued_operations: VecDeque<Box<dyn Operation>>,
}

impl UndoRedo {
	pub fn push_operation<O: Operation>(&mut self, operation: O) {
		self.queued_operations.push_back(Box::new(operation));
	}

	/// Applies all operations that are queued. Any actions that have been undone, but not
	/// subsequently redone, will be lost.
	///
	/// # Possible errors
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
	/// # Possible errors
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
	/// # Possible errors
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
	/// # Possible errors
	/// * [`Error::NoApplicableHistory`] - No operations have been undone since the last time (if any)
	///   queued operations were applied.
	pub fn redo(&mut self) -> Result<&T, Error> {
		// If there are no operations in the history, we have no work to do. Let the caller know.
		if self.undone.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an operation off the end of the undone list...
		//
		// NOTE: Because we just verified that `!self.undone.is_empty()`, we can safely assume
		// this will be a `Some(_)` value.
		let last_undone_operation = self.undone.pop().expect("undone list should not be empty");

		// And add that item to the end of the committed list.
		self.committed.push_back(last_undone_operation);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_undone_operation`, as Rust seems to yell
		// at us if we try.
		let item_ref = self
			.committed
			.back()
			.expect("committed list should not be empty");

		Ok(item_ref)
	}

	/// [TODO: Description]
	///
	/// # Possible errors
	/// * [`Error::NoApplicableHistory`] - There are no operations available to undo.
	pub fn undo(&mut self) -> Result<&T, Error> {
		// If there are no operations in the history, we have no work to do. Let the caller know.
		if self.committed.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an operation off the end of the history...
		//
		// NOTE: Because we just verified that `!self.committed.is_empty()`, we can safely assume
		// this will be a `Some(_)` value.
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

		write!(f, "{}", msg)
	}
}
