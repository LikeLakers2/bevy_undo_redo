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
	///
	/// Operations get a mutable reference to themselves. This allows Operations to store some data
	/// (such as an `Entity`) which can later be used for undoing the operation.
	fn apply(&mut self, commands: &mut Commands);
	/// Queues up the commands needed to undo this operation.
	fn undo(&self, commands: &mut Commands);
}

/// Data representing information about a operation or set of operations.
///
/// This can be obtained through [`Operation::details()`].
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Details {
	/// The type of operation that this is; i.e. "Move object"
	// TODO: Implement an interface to obtain this, rather than just exposing a public variable.
	pub name: String,
}
