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
	/// A list of all operations that have been committed, in the order they were committed.
	history: VecDeque<Box<dyn Operation>>,
	/// A list of all operations that were committed, but have subsequently been undone. Flushing
	/// the queued operations list will clear this list.
	// TODO: Document on the functions that flushing the queued operations list will clear this list
	// "Any operations that have been undone and not subsequently redone will be lost to time"
	//
	// TODO: Maybe combine this into `history` and use a `usize` as a cursor for which ops are
	// committed and which were undone?
	history_undone: VecDeque<Box<dyn Operation>>,
	/// The maximum length of this UndoRedo's history. Any operations past this limit will be
	/// automatically culled from the history.
	history_limit: Option<NonZeroUsize>,
	/// A list of operations that have been pushed to this UndoRedo, but have not been applied to
	/// the World.
	queued_operations: VecDeque<Box<dyn Operation>>,
}

impl UndoRedo {
	pub fn push_operation<O: Operation>(&mut self, operation: O) {
		self.queued_operations.push_back(Box::new(operation));
	}

	/// Applies all operations that are queued.
	pub fn apply_queued_operations(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no queued operations available, we have no work to do. Let the caller know.
		if self.queued_operations.is_empty() {
			return Err(Error::NoActionAvailable);
		}

		let queued_operations = self.queued_operations.drain(..);

		for operation in queued_operations {
			operation.apply(commands);
			self.history.push_back(operation);
		}

		self.history_undone.clear();
		Ok(())
	}

	/// Applies the next queued operation, if any. If there are no queued actions, this does nothing.
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no previously-undone operations available, we have no work to do. Let the
		// caller know.
		if self.history_undone.is_empty() {
			return Err(Error::NoActionAvailable);
		}

		// Otherwise, get the first undone operation...
		//
		// Note: Because we just verified that `!self.history_undone.is_empty()`, we can safely
		// assume this will be a `Some(_)` value.
		let next_queued_action = self
			.history_undone
			.pop_front()
			.expect("history_undone should not be empty");

		// Then have it submit all the commands needed to apply...
		next_queued_action.apply(commands);

		self.history.push_back(next_queued_action);

		Ok(())
	}

	/// Undoes the last committed operation, if any. If there are no committed operations, this does
	/// nothing.
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no operations in the history, we have no work to do. Let the caller know.
		if self.history.is_empty() {
			return Err(Error::NoActionAvailable);
		}

		// Otherwise, pop an operation off the end of the history...
		//
		// NOTE: Because we just verified that `!self.history.is_empty()`, we can safely assume this
		// will be a `Some(_)` value.
		let last_committed_action = self
			.history
			.pop_back()
			.expect("history should not be empty");

		// Then have it submit all the commands needed to undo...
		last_committed_action.undo(commands);

		self.history_undone.push_front(last_committed_action);

		Ok(())
	}
}

#[derive(Debug)]
pub enum Error {
	NoActionAvailable,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::NoActionAvailable => {
				write!(f, "No actions are available to perform this operation")
			}
		}
	}
}
