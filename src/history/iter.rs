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

impl<T> DoubleEndedIterator for Iter<'_, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

/// An iterator over a History's list of committed items.
///
/// Items are returned in order from least-recently committed to most-recently committed.
#[expect(
	clippy::module_name_repetitions,
	reason = "This is an iterator over committed items only."
)]
#[derive(Clone, Debug)]
pub struct CommittedIter<'a, T>(VecDequeIter<'a, T>);

impl<'a, T> CommittedIter<'a, T> {
	/// Returns an instance of `Self`, given iterator over undone items.
	pub(super) const fn new(committed_iter: VecDequeIter<'a, T>) -> Self {
		Self(committed_iter)
	}
}

impl<T> DoubleEndedIterator for CommittedIter<'_, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<T> ExactSizeIterator for CommittedIter<'_, T> {
	fn len(&self) -> usize {
		self.0.len()
	}
}

impl<T> FusedIterator for CommittedIter<'_, T> {}

impl<'a, T> Iterator for CommittedIter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

/// An iterator over a History's list of undone items.
///
/// Items are returned in order from most-recently undone to least-recently undone.
#[expect(
	clippy::module_name_repetitions,
	reason = "This is an iterator over undone items only."
)]
#[derive(Clone, Debug)]
pub struct UndoneIter<'a, T>(Rev<SliceIter<'a, T>>);

impl<'a, T> UndoneIter<'a, T> {
	/// Returns an instance of `Self`, given iterator over undone items.
	pub(super) fn new(undone_iter: SliceIter<'a, T>) -> Self {
		Self(undone_iter.rev())
	}
}

impl<T> DoubleEndedIterator for UndoneIter<'_, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<T> ExactSizeIterator for UndoneIter<'_, T> {
	fn len(&self) -> usize {
		self.0.len()
	}
}

impl<T> FusedIterator for UndoneIter<'_, T> {}

impl<'a, T> Iterator for UndoneIter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

#[cfg(test)]
mod tests {
	use crate::history::History;
	use core::ops::Range;
	use rstest::{fixture, rstest};

	const VALUE_RANGE: Range<u32> = 0..10;
	const COMMITTED_RANGE: Range<u32> = 0..7;
	const UNDONE_RANGE: Range<u32> = 7..10;

	#[fixture]
	#[once]
	fn sample_history() -> History<u32> {
		let mut history = History::new();
		for i in VALUE_RANGE {
			history.push(i);
		}
		for _ in UNDONE_RANGE {
			let _ = history.undo();
		}
		history
	}

	// TODO:
	// * Iter: `Iterator::size_hint`
	// * CommittedIter: `Iterator::size_hint`
	// * UndoneIter: `Iterator::size_hint`

	mod check_iterator {
		use super::*;

		#[rstest]
		#[case::iter(History::iter, VALUE_RANGE)]
		#[case::committed_iter(History::iter_committed, COMMITTED_RANGE)]
		#[case::undone_iter(History::iter_undone, UNDONE_RANGE)]
		fn next<'a, F, I>(
			sample_history: &'a History<u32>,
			#[case] iter_func: F,
			#[case] applicable_range: Range<u32>,
		) where
			F: Fn(&'a History<u32>) -> I,
			I: Iterator<Item = &'a u32>,
		{
			let mut iter = iter_func(sample_history);

			for i in applicable_range {
				assert_eq!(iter.next(), Some(&i));
			}

			assert_eq!(iter.next(), None);
		}
	}

	#[rstest]
	#[case::iter(History::iter, VALUE_RANGE)]
	#[case::committed_iter(History::iter_committed, COMMITTED_RANGE)]
	#[case::undone_iter(History::iter_undone, UNDONE_RANGE)]
	fn check_double_ended_iterator<'a, F, I>(
		sample_history: &'a History<u32>,
		#[case] iter_func: F,
		#[case] applicable_range: Range<u32>,
	) where
		F: Fn(&'a History<u32>) -> I,
		I: DoubleEndedIterator<Item = &'a u32>,
	{
		let mut iter = iter_func(sample_history);

		for i in applicable_range.rev() {
			assert_eq!(iter.next_back(), Some(&i));
		}

		assert_eq!(iter.next(), None);
	}

	#[rstest]
	#[case::committed_iter(History::iter_committed, COMMITTED_RANGE)]
	#[case::undone_iter(History::iter_undone, UNDONE_RANGE)]
	fn check_exact_size_iterator<'a, F, I>(
		sample_history: &'a History<u32>,
		#[case] iter_func: F,
		#[case] applicable_range: Range<u32>,
	) where
		F: Fn(&'a History<u32>) -> I,
		I: ExactSizeIterator<Item = &'a u32>,
	{
		let iter = iter_func(sample_history);
		assert_eq!(iter.len(), applicable_range.len());
	}
}
