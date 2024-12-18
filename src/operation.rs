use core::any::Any;

use bevy_ecs::system::Commands;

pub trait Operation: Send + Sync + 'static {
	fn apply(&self, commands: &mut Commands);
	fn undo(&self, commands: &mut Commands);
}

pub struct Details {
	/// The type of operation that this is; i.e. "Moves an object by a certain amount of pixels"
	op_type: &'static str,

	/// Any additional info provided by the user. The user is responsible for downcasting this to
	/// whatever they need.
	additional_info: Box<dyn Any>,
}

pub struct Set {
	name: String,
	/// The set of operations that this [`OperationSet`] groups together.
	op_list: Vec<Box<dyn Operation>>,
}

impl Set {
	#[must_use]
	pub fn new(name: String) -> Self {
		Self {
			name,
			op_list: vec![],
		}
	}

	pub fn push<O: Operation>(&mut self, operation: O) {
		self.op_list.push(Box::new(operation));
	}
}

impl Operation for Set {
	fn apply(&self, commands: &mut Commands) {
		for op in &self.op_list {
			op.apply(commands);
		}
	}

	fn undo(&self, commands: &mut Commands) {
		let reversed_op_list = self.op_list.iter().rev();
		for op in reversed_op_list {
			op.undo(commands);
		}
	}
}
