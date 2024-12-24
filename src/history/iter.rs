//! Iterators to interact with an instance of [`History`].

use core::{iter::Chain, slice::Iter as SliceIter};
use std::collections::vec_deque::Iter as VecDequeIter;

/// An iterator over all of History's items, both committed and undone.
pub struct Iter<'a, T> {
	/// The iterators over committed items and undone items, chained together.
	_inner: Chain<CommittedIter<'a, T>, UndoneIter<'a, T>>,
}

/// The type of iterator for committed items.
#[expect(
	clippy::module_name_repetitions,
	reason = "This is an iterator over committed items only."
)]
pub type CommittedIter<'a, T> = VecDequeIter<'a, T>;

/// The type of iterator for undone items.
#[expect(
	clippy::module_name_repetitions,
	reason = "This is an iterator over undone items only."
)]
pub type UndoneIter<'a, T> = SliceIter<'a, T>;
