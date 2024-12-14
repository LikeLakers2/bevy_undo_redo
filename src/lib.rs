use bevy_ecs::{
	system::{Commands, Resource},
	world::{CommandQueue, World},
};

use crate::{action::Action, operation::Operation};

pub mod action;
pub mod operation;

#[derive(Default, Resource)]
pub struct UndoRedo {
	/// A list of all actions that have been created in this UndoRedo. This vector contains two
	/// types of actions:
	///
	/// * A committed action. This action has already been applied to the world, and can be undone.
	/// * A queued action. This action has been created, and possibly assigned a set of operations,
	///   but has not been committed to the world.
	action_list: Vec<Action>,
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
	pub fn create_queued_action(&mut self, name: String) -> &mut Action {
		let new_action = Action::new(name);

		self.action_list.push(new_action);

		self.action_list
			.last_mut()
			.expect("queued action list should not be empty, as we just pushed an item")
	}

	pub fn commit_queued_actions(&mut self, world: &mut World) {
		if self.list_cursor == self.action_list.len() {
			return;
		}

		let mut command_queue = CommandQueue::default();
		let mut commands = Commands::new(&mut command_queue, world);

		let queued_actions = self.action_list.iter().skip(self.list_cursor);
		self.list_cursor = self.action_list.len();

		for action in queued_actions {
			action.apply(&mut commands);
		}

		command_queue.apply(world);
	}

	pub fn redo(&mut self, world: &mut World) {
		// If the cursor is already at the end of `self.action_list`, we have no action to apply. In
		// this case, we return without doing anything.
		if self.list_cursor == self.action_list.len() {
			return;
		}

		// Otherwise, initialize a CommandQueue and a Commands...
		let mut command_queue = CommandQueue::default();
		let mut commands = Commands::new(&mut command_queue, world);

		// Then get the next queued action action...
		let next_queued_action = self
			.action_list
			.get(self.list_cursor)
			.expect("next action should exist");

		// Then have it submit all the commands needed to apply...
		next_queued_action.apply(&mut commands);

		// And finally, apply those commands to the world.
		command_queue.apply(world);

		// Now that we've applied the action, the cursor needs to be updated to point to the next
		// queued action (or to the end, if none are queued).
		self.list_cursor += 1;
	}

	pub fn undo(&mut self, world: &mut World) {
		// If the cursor is already at the beginning of `self.action_list`, we have no action to
		// undo. In this case, we return without doing anything.
		if self.list_cursor == 0 {
			return;
		}

		// Otherwise, initialize a CommandQueue and a Commands...
		let mut command_queue = CommandQueue::default();
		let mut commands = Commands::new(&mut command_queue, world);

		// Then, decrement the list cursor...
		self.list_cursor -= 1;

		// Then, use the new cursor position to get the last committed action...
		let last_committed_action = self
			.action_list
			.get(self.list_cursor)
			.expect("previous action should exist");

		// Then have it submit all the commands needed to undo...
		last_committed_action.undo(&mut commands);

		// And finally, apply those commands to the world.
		command_queue.apply(world);
	}
}
