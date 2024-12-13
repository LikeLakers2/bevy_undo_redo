use std::{any::Any, fmt::Debug};

use bevy_ecs::world::CommandQueue;

pub trait Operation: Send + Sync + 'static {
	fn apply(&self) -> CommandQueue;
	fn undo(&self) -> CommandQueue;
}

pub struct Details {
	/// The type of operation that this is; i.e. "Moves an object by a certain amount of pixels"
	op_type: &'static str,

	/// Any additional info provided by the user. The user is responsible for downcasting this to
	/// whatever they need.
	additional_info: Box<dyn Any>,
}
