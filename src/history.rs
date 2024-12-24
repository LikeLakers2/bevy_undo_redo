//! Types related to [`History`], a collection which represents the history of something.
use core::num::NonZeroUsize;

use std::collections::VecDeque;

use crate::error::Error;

/// A collection which holds a set of items that represents the history of something, and acts as a
/// cursor into that set of items.
///
/// Unlike [`UndoRedo`], this struct does not affect a [`World`] when items are pushed to it. It
/// only acts as a pointer into a set of items.
///
/// [`UndoRedo`]: crate::undoredo::UndoRedo
/// [`World`]: bevy_ecs::world::World
// TODO List:
// * `get()`, `get_mut()`
// * `get_limit()`, `set_limit()`
// * `impl<T> IntoIterator for History<T>`
//   * Plus `iter()`, `iter_committed()`, `iter_undone()`
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
	/// Creates a new `History`.
	#[must_use = "History does not store anything on its own - you must push items for it to store."]
	pub const fn new() -> Self {
		Self {
			committed: VecDeque::new(),
			undone: Vec::new(),
			limit: None,
		}
	}
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
		self.truncate_committed_to_limit_plus(1);
		self.committed.push_back(item);
		self.clear_undone();
	}

	/// Marks the last undone item as "committed", and returns a mutable reference to it.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - If there is no history available to redo. This usually
	///   occurs if there haven't been any calls to [`Self::undo()`] since the last time an item was
	///   pushed.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause panics are handled beforehand, returning `Err` instead."
	)]
	pub fn redo(&mut self) -> Result<&mut T, Error> {
		// Attempt to pop an item off the end of the undone list. If we fail, then we have no undone
		// items, and thus cannot perform the redo operation - in which case, we should return an
		// error.
		let Some(last_undone_item) = self.undone.pop() else {
			return Err(Error::NoApplicableHistory);
		};

		// And add that item to the end of the committed list.
		self.committed.push_back(last_undone_item);

		// Finally, return a mutable reference to the item we just moved between lists.
		//
		// NOTE: We unfortunately can't just return `&last_undone_item`, as Rust seems to yell at us
		// if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.committed`.
		let item_ref = self
			.committed
			.back_mut()
			.expect("committed list should not be empty");

		Ok(item_ref)
	}

	/// Marks the last committed item as "undone", and returns a mutable reference to it.
	///
	/// # Errors
	/// * [`Error::NoApplicableHistory`] - If there is no history available to undo.
	#[expect(
		clippy::missing_panics_doc,
		reason = "This function cannot panic under normal circumstances, as the conditions which would cause panics are handled beforehand, returning `Err` instead."
	)]
	pub fn undo(&mut self) -> Result<&mut T, Error> {
		// Attempt to pop an item off the end of the history. If we fail, then we have no committed
		// items, and thus cannot perform the undo operation - in which case, we should return an
		// error.
		let Some(last_committed_item) = self.committed.pop_back() else {
			return Err(Error::NoApplicableHistory);
		};

		// And add that item to the end of the undone list.
		self.undone.push(last_committed_item);

		// Finally, return a mutable reference to the item we just moved between lists.
		//
		// NOTE: We unfortunately can't just return `&last_committed_item`, as Rust seems to yell at
		// us if we try.
		//
		// NOTE: This cannot panic, as we've just pushed an item to `self.undone`.
		let item_ref = self
			.undone
			.last_mut()
			.expect("undone list should not be empty");

		Ok(item_ref)
	}
}

/// Private items. This helps keep the secondary side bar in vscode cleaner, by separating this
/// module into public and private items.
impl<T> History<T> {
	/// Truncates `self.committed` such that it only contains `self.limit` items.
	///
	/// This also takes a parameter `plus`, which causes the truncation to act as if
	/// `self.committed` had `plus` more items. This is useful if you're about to push an item, as
	/// it ensures we'll never have more than `self.limit` items in `self.committed`.
	fn truncate_committed_to_limit_plus(&mut self, plus: usize) {
		if let Some(limit) = self.limit {
			// Transform this from a `NonZero<usize>` to a `usize`.
			let limit = limit.get();

			// Calculate how many items we'd have after the upcoming push, if we weren't limited.
			let len_after_push = self.committed.len() + plus;

			// Then, calculate how many items to remove, saturating at 0.
			let count_to_remove = len_after_push.saturating_sub(limit);

			// Then, drain that many items out of the beginning of the committed list.
			//
			// Technically speaking, draining creates an iterator that moves items off the list,
			// rather than directly removing items. However, dropping that iterator gives those
			// drained items nowhere to go - and so they too will be dropped. In essence, this is
			// like a theoretical `truncate_front()` function on `VecDeque`.
			self.committed.drain(0..count_to_remove);
		}
	}
}

// Manually impl Default, to avoid putting a bound on T.
impl<T> Default for History<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Extend<T> for History<T> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		match self.limit {
			None => {
				// In this case, we can just defer to VecDeque's Extend impl.
				self.committed.extend(iter);
				self.clear_undone();
			}
			Some(_limit) => {
				// In this case, we need to account for the limit - and ideally, do so without ever
				// having more than `limit` items in `self.committed`, even for a moment.
				//
				// TODO: Depending on how Rust optimizes this code, this could be made more
				// efficient.
				for item in iter {
					self.push(item);
				}
			}
		}
	}
}

impl<T> FromIterator<T> for History<T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		Self {
			committed: FromIterator::from_iter(iter),
			..Default::default()
		}
	}
}
