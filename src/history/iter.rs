//! Iterators to interact with an instance of [`History`].

use core::{
	iter::{Chain, FusedIterator, Rev},
	slice::Iter as SliceIter,
};
use std::collections::vec_deque::Iter as VecDequeIter;

/// An iterator over all of History's items, both committed and undone.
///
/// Committed items are returned first, in order from least-recently committed to most-recently
/// committed. Then, undone items are returned, in order from most-recently undone to least-recently
/// undone.
#[derive(Clone, Debug)]
pub struct Iter<'a, T>(Chain<CommittedIter<'a, T>, UndoneIter<'a, T>>);

impl<'a, T> Iter<'a, T> {
	/// Returns an instance of `Self`, given an iterator over committed items, and an iterator over
	/// undone items.
	pub(super) fn new(
		committed_iter: CommittedIter<'a, T>,
		undone_iter: UndoneIter<'a, T>,
	) -> Self {
		Self(committed_iter.chain(undone_iter))
	}
}

/// The type of iterator for committed items.
///
/// Items are returned in order from least-recently committed to most-recently committed.
// TODO: Replace this with an actual iterator, so we don't expose the internal type.
#[expect(
	clippy::module_name_repetitions,
	reason = "This is an iterator over committed items only."
)]
pub type CommittedIter<'a, T> = VecDequeIter<'a, T>;

/// The type of iterator for undone items.
///
/// Items are returned in order from most-recently undone to least-recently undone.
// TODO: Replace this with an actual iterator, so we don't expose the internal type.
#[expect(
	clippy::module_name_repetitions,
	reason = "This is an iterator over undone items only."
)]
pub type UndoneIter<'a, T> = Rev<SliceIter<'a, T>>;

impl<T> DoubleEndedIterator for Iter<'_, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl<T> FusedIterator for Iter<'_, T> {}
