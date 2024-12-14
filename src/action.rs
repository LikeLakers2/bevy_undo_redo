use bevy_ecs::system::Commands;

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
	fn apply(&self, commands: &mut Commands) {
		for op in &self.op_list {
			op.apply(commands);
		}
	}

	fn undo(&self, commands: &mut Commands) {
		for op in &self.op_list {
			op.undo(commands);
		}
	}
}
