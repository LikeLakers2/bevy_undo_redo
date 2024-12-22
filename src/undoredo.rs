use std::collections::VecDeque;

use bevy_ecs::system::{Commands, Resource};

use crate::{error::Error, history::History, operation::Operation};

/// A high-level interface for implementing undo/redo functionality.
#[derive(Default, Resource)]
pub struct UndoRedo {
	/// The collection which manages the list of committed operations, and acts as a pointer into
	/// that set of items.
	history: History<Box<dyn Operation>>,
	/// A list of operations that have been pushed to this [`UndoRedo`], but have not been applied
	/// to the World.
	queued_operations: VecDeque<Box<dyn Operation>>,
}

impl UndoRedo {
	/// Pushes an operation into the list of queued operations.
	///
	/// After pushing one or more operations, call [`Self::apply_queued_operations()`] to apply the
	/// operation(s) to the [`World`].
	///
	/// [`World`]: bevy_ecs::world::World
	pub fn push_operation<O: Operation>(&mut self, operation: O) {
		self.queued_operations.push_back(Box::new(operation));
	}

	/// Applies all operations that are queued. Any actions that have been undone, but not
	/// subsequently redone, will be lost.
	///
	/// # Errors
	/// * [`Error::NoQueuedOperations`] - There are no queued operations available to apply.
	pub fn apply_queued_operations(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no queued operations available, we have no work to do. Let the caller know.
		if self.queued_operations.is_empty() {
			return Err(Error::NoQueuedOperations);
		}

		let queued_operations = self.queued_operations.drain(..);

		for operation in queued_operations {
			operation.apply(commands);
			self.history.push(operation);
		}

		Ok(())
	}

	/// Applies the last undone operation, if any.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - No operations have been undone since the last time (if any)
	///   queued operations were applied.
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		let item = self.history.redo()?;

		// Submit all the commands needed to apply...
		item.apply(commands);

		Ok(())
	}

	/// Undoes the last committed operation, if any.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - There are no operations available to undo.
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		let item = self.history.undo()?;

		// Submit all the commands needed to undo...
		item.undo(commands);

		Ok(())
	}
}
