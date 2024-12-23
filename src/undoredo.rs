//! A high-level interface for implementing undo/redo functionality.
use std::collections::VecDeque;

use bevy_ecs::system::{Commands, ResMut, Resource};

use crate::{error::Error, history::History, operation::Operation};

/// A high-level interface for implementing undo/redo functionality.
///
/// If you've inserted an `UndoRedo` as a `Resource` into a world and want to undo/redo an
/// operation, consider using [`CommandsUndoRedoExt`], which negates the need to supply a `Commands`
/// yourself.
///
/// # Operation States
/// Operations are given one of three states:
///
/// * **Queued** - The operation has been pushed to the `UndoRedo`, and is ready to be applied to
///   the world.
/// * **Applied** - The operation's effects have been applied to the world, or will be applied the
///   next time [`Commands`] are applied. You can undo these operations.
/// * **Undone** - The operation's effects have been reversed , or will be the next time
///   [`Commands`] are applied). They can be redone, but all undone operations are lost the next
///   time an operation is marked as **Committed**.
///
/// [`CommandsUndoRedoExt`]: crate::extensions::CommandsUndoRedoExt
#[derive(Default, Resource)]
pub struct UndoRedo {
	/// The collection which manages the list of applied and undone operations, and acts as a
	/// pointer into that set of items.
	history: History<Box<dyn Operation>>,
	/// A list of operations that have been pushed to this [`UndoRedo`], but have not been applied
	/// to the World.
	queued_operations: VecDeque<Box<dyn Operation>>,
}

impl UndoRedo {
	/// Clears all stored operations, including those that are still queued.
	pub fn clear(&mut self) {
		self.history.clear();
		self.queued_operations.clear();
	}

	/// Clears the list of queued operations.
	pub fn clear_queue(&mut self) {
		self.queued_operations.clear();
	}

	/// Pushes an operation into the list of queued operations. Queued operations are those that are
	/// ready to be applied later.
	///
	/// After pushing one or more operations, call [`Self::apply_queue()`] to apply the operation(s)
	/// to the [`World`].
	///
	/// # See Also
	/// * [`Self::push_and_apply()`] - Pushes an operation, skipping the queue such that it will be
	///   applied ASAP.
	///
	/// [`World`]: bevy_ecs::world::World
	pub fn push_to_queue<O: Operation>(&mut self, operation: O) {
		self.queued_operations.push_back(Box::new(operation));
	}

	/// Queues up the commands needed to apply all queued operations, and moves those queued
	/// operations to the list of applied operations.
	///
	/// Additionally, any operations which have been undone, but not subsequently redone, will be
	/// lost when calling this.
	///
	/// # Errors
	/// * [`Error::NoQueuedOperations`] - There are no queued operations available to apply.
	// TODO: This should probably be called by a built-in system.
	pub fn apply_queue(&mut self, commands: &mut Commands) -> Result<(), Error> {
		// If there are no queued operations available, we have no work to do. Let the caller know.
		if self.queued_operations.is_empty() {
			return Err(Error::NoQueuedOperations);
		}

		let queued_operations = self.queued_operations.drain(..);

		for mut operation in queued_operations {
			operation.apply(commands);
			self.history.push(operation);
		}

		Ok(())
	}

	/// Queues up the commands needed to apply `operation`, then pushes `operation` to the list of
	/// applied operations.
	///
	/// The list of queued operations is untouched when calling this. However, undone operations
	/// which have not been subsequently redone *will* be lost, as with [`Self::apply_queue()`].
	///
	/// # Errors
	/// None as of yet.
	///
	/// # See Also
	/// * [`Self::push_to_queue()`] - Pushes items to a queue, to be applied later all at once.
	pub fn push_and_apply<O: Operation>(
		&mut self,
		operation: O,
		commands: &mut Commands,
	) -> Result<(), Error> {
		let mut operation = Box::new(operation);
		operation.apply(commands);
		self.history.push(operation);
		Ok(())
	}

	/// Applies the last undone operation, if any.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - No operations have been undone since the last time (if any)
	///   queued operations were applied.
	///
	/// # See Also
	/// * [`CommandsUndoRedoExt::redo()`] - Queues up a call to this method on the world's
	///   `UndoRedo` resource.
	///
	/// [`CommandsUndoRedoExt::redo()`]: crate::extensions::CommandsUndoRedoExt::redo()
	pub fn redo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		let item = self.history.redo()?;

		// Submit all the commands needed to apply...
		item.apply(commands);

		Ok(())
	}

	/// Undoes the last applied operation, if any.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - There are no operations available to undo.
	///
	/// # See Also
	/// * [`CommandsUndoRedoExt::redo()`] - Queues up a call to this method on the world's
	///   `UndoRedo` resource.
	///
	/// [`CommandsUndoRedoExt::redo()`]: crate::extensions::CommandsUndoRedoExt::redo()
	pub fn undo(&mut self, commands: &mut Commands) -> Result<(), Error> {
		let item = self.history.undo()?;

		// Submit all the commands needed to undo...
		item.undo(commands);

		Ok(())
	}
}

/// Applies any queued operations when this system is run.
pub fn apply_queued_operations(mut undoredo: ResMut<UndoRedo>, mut commands: Commands) {
	// We intentionally ignore any result, as we don't care how much work was done.
	let _ = undoredo.apply_queue(&mut commands);
}
