//! A collection of [`Operation`]s, used to group them together and treat them as one operation.
use bevy_ecs::{system::Commands, world::{Command, CommandQueue, World}};

use crate::operation::{Details, Operation};

/// A collection of [`Operation`]s, used to group them together and treat them as one operation.
pub struct OperationGroup {
	/// A descriptor for this set of Operations.
	details: Details,
	/// The set of operations that this [`OperationSet`] groups together.
	op_list: Vec<Box<dyn Operation>>,
}

impl OperationGroup {
	/// Creates a new [`Set`].
	#[must_use]
	pub fn new(details: Details) -> Self {
		Self {
			details,
			op_list: vec![],
		}
	}

	/// Pushes an operation into this [`Set`]. Operations will be applied in the order they were
	/// pushed, and undone in reverse order.
	pub fn push<O: Operation>(&mut self, operation: O) {
		self.op_list.push(Box::new(operation));
	}
}

impl Command for OperationGroup {
	fn apply(mut self, world: &mut World) {
		let mut command_queue = CommandQueue::default();
		let mut commands = Commands::new(&mut command_queue, world);

		Operation::apply(&mut self, &mut commands);

		command_queue.apply(world);
	}
}

impl Operation for OperationGroup {
	fn details(&self) -> Details {
		self.details.clone()
	}

	fn apply(&mut self, commands: &mut Commands) {
		for op in &mut self.op_list {
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
