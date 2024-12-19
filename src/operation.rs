//! Types and traits for implementing and handling [`Operation`]s.

use bevy_ecs::system::Commands;

/// An action or sequence of commands which can later be undone.
///
/// This can be thought of as an "undoable [`Command`]". In fact, in many cases, an `Operation` will
/// itself also implement `Command`.
///
/// [`Command`]: bevy_ecs::world::Command
pub trait Operation: Send + Sync + 'static {
	/// Returns a list of details related to this operation.
	fn details(&self) -> Details;

	/// Queues up the commands needed to apply this operation to the World.
	fn apply(&self, commands: &mut Commands);
	/// Queues up the commands needed to undo this operation.
	fn undo(&self, commands: &mut Commands);
}

/// Data representing information about a operation or set of operations.
///
/// This can be obtained through [`Operation::details()`].
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Details {
	/// The type of operation that this is; i.e. "Move object"
	// TODO: Implement an interface to obtain this, rather than just exposing a public variable.
	pub name: String,
}

/// A collection of operations, used to group operations together.
pub struct Set {
	/// A descriptor for this set of Operations.
	name: String,
	/// The set of operations that this [`OperationSet`] groups together.
	op_list: Vec<Box<dyn Operation>>,
}

impl Set {
	/// Creates a new [`Set`].
	#[must_use]
	pub fn new(name: String) -> Self {
		Self {
			name,
			op_list: vec![],
		}
	}

	/// Pushes an operation into this [`Set`]. Operations will be applied in the order they were
	/// pushed, and undone in reverse order.
	pub fn push<O: Operation>(&mut self, operation: O) {
		self.op_list.push(Box::new(operation));
	}
}

impl Operation for Set {
	fn details(&self) -> Details {
		Details {
			name: self.name.clone(),
		}
	}

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
