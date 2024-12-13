use bevy_ecs::system::Resource;

use crate::operation::Operation;

pub mod operation;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Resource)]
pub struct UndoRedo<O: Operation> {
	/// The list of stored operations that either have already been taken, or will be taken once
	/// committed.
	/// 
	/// TODO: This list should probably have a limit somewhere.
	op_list: Vec<O>,
	/// Where we are in `op_list`. Operations that have already been applied will be before this
	/// index, and operations that have yet to be applied will be after.
	cursor_pos: usize,
}
