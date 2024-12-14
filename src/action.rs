use bevy_ecs::world::CommandQueue;

use crate::operation::Operation;

pub struct Action {
	name: String,
	/// The list of operations this action takes.
	op_list: Vec<Box<dyn Operation>>,
}

impl Action {
	pub(crate) fn new(name: String) -> Self {
		Self {
			name,
			op_list: vec![],
		}
	}

	pub fn push<O: Operation>(&mut self, operation: O) {
		self.op_list.push(Box::new(operation))
	}
}

impl Operation for Action {
	fn get_apply_command(&self) -> CommandQueue {
		let mut queue = CommandQueue::default();
		for op in &self.op_list {
			let mut op_queue = op.get_apply_command();
			queue.append(&mut op_queue);
		}
		queue
	}

	fn get_undo_command(&self) -> CommandQueue {
		let mut queue = CommandQueue::default();
		for op in &self.op_list {
			let mut op_queue = op.get_undo_command();
			queue.append(&mut op_queue);
		}
		queue
	}
}
