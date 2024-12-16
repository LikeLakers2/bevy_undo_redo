use std::{
	collections::VecDeque,
	fmt::{Display, Formatter, Result as FmtResult},
};

use bevy_ecs::system::{Commands, Resource};

use crate::operation::Operation;

pub mod operation;

#[derive(Default, Resource)]
pub struct UndoRedo {
	/// A list of all operations that have been pushed to this UndoRedo.
	pushed_operations: VecDeque<Box<dyn Operation>>,
	/// An index that represents a cursor into `self.pushed_operations`.
	/// 
	/// Operations before this cursor (called "committed operations") have all been previously
	/// applied to the World, and can be undone. Operations after this cursor (called "queued
	/// operations") are waiting to be committed to the World, and may have previously been undone.
	list_cursor: usize,
}

impl UndoRedo {
	pub fn push_operation<O: Operation>(&mut self, operation: O) {
		self.pushed_operations.push_back(Box::new(operation));
	}

	/// Applies all operations that are queued.
	pub fn apply_queued_operation(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If the cursor is already at the end of `self.pushed_operations`, we have no actions to apply.
		if self.list_cursor == self.pushed_operations.len() {
			return Err(Error::NoActionAvailable);
		}

		let queued_actions = self.pushed_operations.iter().skip(self.list_cursor);

		for action in queued_actions {
			action.apply(commands);
		}

		self.list_cursor = self.pushed_operations.len();

		Ok(())
	}

	/// Applies the next queued operation, if any. If there are no queued actions, this does nothing.
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If the cursor is already at the end of `self.pushed_operations`, we have no operation to
		// apply. In this case, we return an error describing as such.
		if self.list_cursor == self.pushed_operations.len() {
			return Err(Error::NoActionAvailable);
		}

		// Otherwise, get the next queued operation...
		//
		// Note: Because we just verified that we're not at the end of the pushed operations list,
		// we can safely assume this will be a `Some(_)` value.
		let next_queued_action = self
			.pushed_operations
			.get(self.list_cursor)
			.expect("next action should exist");

		// Then have it submit all the commands needed to apply...
		next_queued_action.apply(commands);

		// Now that we've applied the operation, the cursor needs to be updated to point to the next
		// queued operation (or to the end, if none are queued).
		self.list_cursor += 1;

		Ok(())
	}

	/// Undoes the last committed operation, if any. If there are no committed operations, this does
	/// nothing.
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If the cursor is already at the beginning of `self.pushed_operations`, we have no action
		// to undo.
		if self.list_cursor == 0 {
			return Err(Error::NoActionAvailable);
		}

		// Then, decrement the list cursor...
		self.list_cursor -= 1;

		// Then, use the new cursor position to get the last committed operation...
		//
		// Note: Because we just verified that we're not at the beginning of the pushed operations
		// list, we can safely assume this will be a `Some(_)` value.
		let last_committed_action = self
			.pushed_operations
			.get(self.list_cursor)
			.expect("previous action should exist");

		// Then have it submit all the commands needed to undo...
		last_committed_action.undo(commands);

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
