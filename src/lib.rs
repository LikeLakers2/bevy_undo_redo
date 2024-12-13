use std::any::Any;

use bevy_ecs::system::Resource;

use crate::operation::Operation;

pub mod operation;

#[derive(Default, Resource)]
pub struct UndoRedo {
	past_actions: Vec<Action>,
	queued_actions: Vec<Action>,
}

impl UndoRedo {
	pub fn create_action(&mut self, name: String) -> &mut Action {
		let new_action = Action {
			name,
			op_list: vec![],
		};

		self.queued_actions.push(new_action);

		self.queued_actions
			.last_mut()
			.expect("queued action list should not be empty, as we just pushed an item")
	}
}

pub struct Action {
	name: String,
	/// The list of operations this action takes.
	op_list: Vec<Box<dyn Operation>>,
}

impl Action {
	pub fn push<O: Operation>(&mut self, operation: O) {
		self.op_list.push(Box::new(operation))
	}
}
