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
	history: History,
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
	/// # Returns
	/// * [`Error::NoWorkAvailable`] - There are no queued operations available to apply.
	pub fn apply_queued_operations(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no queued operations available, we have no work to do. Let the caller know.
		if self.queued_operations.is_empty() {
			return Err(Error::NoWorkAvailable);
		}

		let queued_operations = self.queued_operations.drain(..);

		for operation in queued_operations {
			operation.apply(commands);
			self.history.committed.push_back(operation);
		}

		self.history.undone.clear();
		Ok(())
	}

	/// Applies the last undone operation, if any.
	///
	/// # Returns
	/// * [`Error::NoApplicableHistory`] - No operations have been undone since the last time (if any)
	///   queued operations were applied.
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no previously-undone operations available, we have no work to do. Let the
		// caller know.
		if self.history.undone.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, get the first undone operation...
		//
		// Note: Because we just verified that `!self.history_undone.is_empty()`, we can safely
		// assume this will be a `Some(_)` value.
		let last_undone_action = self
			.history
			.undone
			.pop_front()
			.expect("history_undone should not be empty");

		// Then have it submit all the commands needed to apply...
		last_undone_action.apply(commands);

		self.history.committed.push_back(last_undone_action);

		Ok(())
	}

	/// Undoes the last committed operation, if any.
	///
	/// # Returns
	/// * [`Error::NoApplicableHistory`] - There are no operations available to undo.
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no operations in the history, we have no work to do. Let the caller know.
		if self.history.committed.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an operation off the end of the history...
		//
		// NOTE: Because we just verified that `!self.history.is_empty()`, we can safely assume this
		// will be a `Some(_)` value.
		let last_committed_action = self
			.history
			.committed
			.pop_back()
			.expect("history should not be empty");

		// Then have it submit all the commands needed to undo...
		last_committed_action.undo(commands);

		self.history.undone.push_front(last_committed_action);

		Ok(())
	}
}

#[derive(Default)]
pub struct History {
	/// A list of all operations that have been committed, in the order they were committed.
	committed: VecDeque<Box<dyn Operation>>,
	/// A list of all operations that were committed, but have subsequently been undone. Flushing
	/// the queued operations list will clear this list.
	// TODO: Document on the functions that flushing the queued operations list will clear this list
	// "Any operations that have been undone and not subsequently redone will be lost to time"
	//
	// TODO: Maybe combine this into `history` and use a `usize` as a cursor for which ops are
	// committed and which were undone?
	undone: VecDeque<Box<dyn Operation>>,
	/// The maximum length of this history. Any operations past this limit will be automatically
	/// culled.
	limit: Option<NonZeroUsize>,
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
