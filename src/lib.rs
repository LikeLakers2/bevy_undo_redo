use bevy_ecs::world::Command;

pub trait Operation {
	fn apply(&self) -> impl Command;
	fn undo(&self) -> impl Command;
}
