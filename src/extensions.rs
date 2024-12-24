//! Extensions for various Bevy types, to make using undo/redo functionality more friendly.

use bevy_ecs::{
	system::Commands,
	world::{Command, Mut, World},
};

use crate::{error::Error as HistoryError, undoredo::UndoRedo};

/// Extension trait for [`Commands`], adding undo/redo-related commands.
pub trait CommandsUndoRedoExt {
	/// Pushes a [`Command`] to the queue for performing an undo using the world's [`UndoRedo`]
	/// resource.
	///
	/// # Panics
	/// The command will panic if no [`UndoRedo`] resource has been inserted.
	fn undo(&mut self);
	/// Pushes a [`Command`] to the queue for performing an redo using the world's [`UndoRedo`]
	/// resource.
	///
	/// # Panics
	/// The command will panic if no [`UndoRedo`] resource has been inserted.
	fn redo(&mut self);
}

impl CommandsUndoRedoExt for Commands<'_, '_> {
	fn undo(&mut self) {
		self.queue(PerformUndo);
	}

	fn redo(&mut self) {
		self.queue(PerformRedo);
	}
}

/// Grabs the `UndoRedo` resource from the world, creates a `Commands`, and then calls a given
/// closure with both.
///
/// # Panics
/// Panics if no [`UndoRedo`] resource has been inserted.
fn use_undoredo_with_commands(
	world: &mut World,
	f: impl FnOnce(&mut UndoRedo, &mut Commands) -> Result<(), HistoryError>,
) -> Result<(), HistoryError> {
	// We have to use a resource scope here, as we also need to create a new `Commands` - but
	// attempting to do so while UndoRedo is still in the World would result in us violating
	// Rust's aliasing rules.
	world.resource_scope(|world, mut undoredo: Mut<UndoRedo>| {
		let mut commands = world.commands();
		f(&mut undoredo, &mut commands)
	})
}

/// Command that performs an undo using the world's [`UndoRedo`] resource.
pub struct PerformUndo;

impl Command for PerformUndo {
	fn apply(self, world: &mut World) {
		let _ = self::use_undoredo_with_commands(world, UndoRedo::undo);
	}
}

/// Command that performs a redo using the world's [`UndoRedo`] resource.
pub struct PerformRedo;

impl Command for PerformRedo {
	fn apply(self, world: &mut World) {
		let _ = self::use_undoredo_with_commands(world, UndoRedo::redo);
	}
}
