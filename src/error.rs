//! The error type for history-type operations.
use core::fmt::{Display, Formatter, Result as FmtResult};

/// The error type for history-type operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	/// There is no applicable history available for this operation.
	NoApplicableHistory,
	/// There is no queued operation available to apply.
	NoQueuedOperations,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let msg = match self {
			Self::NoApplicableHistory => {
				"No applicable history available to perform this operation"
			}
			Self::NoQueuedOperations => "No operation available to apply",
		};

		write!(f, "{msg}")
	}
}
