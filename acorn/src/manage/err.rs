use std::io;

use thiserror::Error;

use crate::{
	disk::{storage, wal},
	id::PageId,
};

#[derive(Debug, Error)]
pub(crate) enum Error {
	#[error("Segment {0} is corrupted")]
	CorruptedSegment(u32),

	#[error("You've somehow reached acorn's internal size limit limit, which is 4 exibytes, assuming you're using the default page size. Great job! Your database is now broken. ¯\\_(ツ)_/¯")]
	SizeLimitReached,

	#[error(transparent)]
	Disk(#[from] storage::Error),

	#[error("Failed to read from WAL: {0}")]
	WalRead(#[from] wal::ReadError),

	#[error("Failed to write to WAL: {0}")]
	WalWrite(io::Error),

	#[error("The WAL is corrupted")]
	WalCorrupted,

	#[error("B-Tree index page {0} is corrupted")]
	CorruptedBTreeIndex(PageId),
}
