use std::{
	collections::VecDeque,
	fmt::{Display, Formatter, Result as FmtResult},
};

use bevy_ecs::system::{Commands, Resource};

use crate::operation::Operation;

pub mod operation;

#[derive(Default, Resource)]
pub struct UndoRedo {
	/// A list of all actions that have been created in this UndoRedo. This vector contains two
	/// types of actions:
	///
	/// * A committed action. This action has already been applied to the world, and can be undone.
	/// * A queued action. This action has been created, and possibly assigned a set of operations,
	///   but has not been committed to the world.
	action_list: VecDeque<Box<dyn Operation>>,
	/// The first index in `action_list` to be a queued action.
	///
	/// Actions before this index are all committed actions, and actions at or after this index are
	/// all queued actions.
	///
	/// This is used over two `Vec<Action>`s as a small optimization - it requires less processing
	/// to change a `usize`, than it does to remove a
	list_cursor: usize,
}

impl UndoRedo {
	pub fn push_operation<O: Operation>(&mut self, operation: O) {
		self.action_list.push_back(Box::new(operation));
	}

	/// Applies all actions that are queued.
	pub fn apply_queued_operation(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If the cursor is already at the end of `self.action_list`, we have no actions to apply.
		if self.list_cursor == self.action_list.len() {
			return Err(Error::NoActionAvailable);
		}

		let queued_actions = self.action_list.iter().skip(self.list_cursor);

		for action in queued_actions {
			action.apply(commands);
		}

		self.list_cursor = self.action_list.len();

		Ok(())
	}

	/// Applies the next queued action, if any. If there are no queued actions, this does nothing.
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If the cursor is already at the end of `self.action_list`, we have no action to apply. In
		// this case, we return an error describing as such.
		if self.list_cursor == self.action_list.len() {
			return Err(Error::NoActionAvailable);
		}

		// Then get the next queued action action...
		//
		// Note: Because we just verified that we're not at the end of the action list, we can
		// safely assume this will be a `Some(_)` value.
		let next_queued_action = self
			.action_list
			.get(self.list_cursor)
			.expect("next action should exist");

		// Then have it submit all the commands needed to apply...
		next_queued_action.apply(commands);

		// Now that we've applied the action, the cursor needs to be updated to point to the next
		// queued action (or to the end, if none are queued).
		self.list_cursor += 1;

		Ok(())
	}

	/// Undoes the last committed action, if any. If there are no committed actions, this does
	/// nothing.
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If the cursor is already at the beginning of `self.action_list`, we have no action to
		// undo.
		if self.list_cursor == 0 {
			return Err(Error::NoActionAvailable);
		}

		// Then, decrement the list cursor...
		self.list_cursor -= 1;

		// Then, use the new cursor position to get the last committed action...
		//
		// Note: Because we just verified that we're not at the beginning of the action list, we can
		// safely assume this will be a `Some(_)` value.
		let last_committed_action = self
			.action_list
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
