use std::any::Any;

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
