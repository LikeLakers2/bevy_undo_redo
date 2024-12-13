use std::fmt::Debug;

use bevy_ecs::world::CommandQueue;

pub trait Operation: Debug + Send + Sync + 'static {
	fn apply(&self) -> CommandQueue;
	fn undo(&self) -> CommandQueue;
}
