//! Extensions for various Bevy types, to make using undo/redo functionality more friendly.

use bevy_ecs::{
	system::Commands,
	world::{Mut, World},
};

use crate::undoredo::UndoRedo;

/// Extension trait for [`Commands`], adding undo/redo-related commands.
pub trait CommandsUndoRedoExt {
	/// Pushes a [`Command`] to the queue for performing an undo using the world's [`UndoRedo`]
	/// resource.
	///
	/// # Panics
	/// The command will panic if no [`UndoRedo`] has been registered as a resource.
	///
	/// [`Command`]: bevy_ecs::world::Command
	fn undo(&mut self);
	/// Pushes a [`Command`] to the queue for performing an redo using the world's [`UndoRedo`]
	/// resource.
	///
	/// # Panics
	/// The command will panic if no [`UndoRedo`] has been registered as a resource.
	///
	/// [`Command`]: bevy_ecs::world::Command
	fn redo(&mut self);
}

impl CommandsUndoRedoExt for Commands<'_, '_> {
	fn undo(&mut self) {
		self.queue(move |world: &mut World| {
			world.resource_scope(|world, mut undoredo: Mut<UndoRedo>| {
				let mut commands = world.commands();
				// We intentionally ignore the result.
				// Though perhaps in the future we could write an event with the result?
				let _ = undoredo.undo(&mut commands);
			});
		});
	}

	fn redo(&mut self) {
		self.queue(move |world: &mut World| {
			world.resource_scope(|world, mut undoredo: Mut<UndoRedo>| {
				let mut commands = world.commands();
				// We intentionally ignore the result.
				// Though perhaps in the future we could write an event with the result?
				let _ = undoredo.redo(&mut commands);
			});
		});
	}
}
