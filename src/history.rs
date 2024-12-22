use core::num::NonZeroUsize;

use std::collections::VecDeque;

use crate::error::Error;

/// A collection which holds a set of items that represents the history of something, and acts as a
/// cursor into that set of items.
///
/// Unlike [`UndoRedo`], this struct does not affect a [`World`] when items are pushed to it. It
/// only acts as a pointer into a set of items.
///
/// [`World`]: bevy_ecs::world::World
pub struct History<T> {
	/// A list of all items that have been committed, in the order they were committed. The
	/// front-most item is the oldest committed item, and the back-most item is the newest committed
	/// item.
	committed: VecDeque<T>,
	/// A list of all items that were committed, but have subsequently been undone. Items at the end
	/// of the list are the most recently undone.
	// NOTE: Because we only care about items at one end of this list, we use a Vec rather than a
	// VecDeque, to gain a small amount of free performance.
	undone: Vec<T>,
	/// The maximum length of this history. Any committed items past this limit will be
	/// automatically culled the next time an item is pushed.
	pub limit: Option<NonZeroUsize>,
}

impl<T> History<T> {
	/// Clears the history of all items.
	pub fn clear(&mut self) {
		self.committed.clear();
		self.undone.clear();
	}
	
	/// Clears the history of all undone items. This prevents [`History::redo()`] from re-applying
	/// any such items.
	pub fn clear_undone(&mut self) {
		self.undone.clear();
	}
	
	/// Pushes an item to the history. This also clears the undone list.
	///
	/// If a history limit is set, any items past the limit will be removed, plus one more to make
	/// space for the item being pushed.
	pub fn push(&mut self, item: T) {
		if let Some(limit) = self.limit {
			// If we have more committed items than the limit, remove items from the committed
			// list until we're at the limit. Then, remove one more to make space for the item we're
			// about to push.
			while limit.get() <= self.committed.len() {
				self.committed.pop_front();
			}
		}

		self.committed.push_back(item);
		self.clear_undone();
	}

	/// Marks the last undone item as "committed", and returns a reference to it.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - If there is no history available to redo. This usually
	///   occurs if there haven't been any calls to [`Self::undo()`] since the last time an item was
	///   pushed.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause those panics are handled beforehand."
	)]
	pub fn redo(&mut self) -> Result<&T, Error> {
		// If there are no items in the history, we have no work to do. Let the caller know.
		if self.undone.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an item off the end of the undone list...
		//
		// NOTE: This cannot panic, as we just verified that `!self.undone.is_empty()`.
		let last_undone_item = self.undone.pop().expect("undone list should not be empty");

		// And add that item to the end of the committed list.
		self.committed.push_back(last_undone_item);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_undone_item`, as Rust seems to yell at us
		// if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.committed`.
		let item_ref = self
			.committed
			.back()
			.expect("committed list should not be empty");

		Ok(item_ref)
	}

	/// Marks the last committed item as "undone", and returns a reference to it.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - If there is no history available to undo.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause those panics are handled beforehand."
	)]
	pub fn undo(&mut self) -> Result<&T, Error> {
		// If there are no items in the history, we have no work to do. Let the caller know.
		if self.committed.is_empty() {
			return Err(Error::NoApplicableHistory);
		}

		// Otherwise, pop an item off the end of the history...
		//
		// NOTE: This cannot panic, as we just verified that `!self.committed.is_empty()`.
		let last_committed_item = self
			.committed
			.pop_back()
			.expect("committed list should not be empty");

		// And add that item to the end of the undone list.
		self.undone.push(last_committed_item);

		// Finally, return a reference to the item we just moved between list.
		//
		// NOTE: We unfortunately can't just return `&last_committed_item`, as Rust seems to yell at
		// us if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.undone`.
		let item_ref = self.undone.last().expect("undone list should not be empty");

		Ok(item_ref)
	}
}

// Manually impl Default, to avoid putting a bound on T.
impl<T> Default for History<T> {
	fn default() -> Self {
		Self {
			committed: VecDeque::new(),
			undone: Vec::new(),
			limit: None,
		}
	}
}
