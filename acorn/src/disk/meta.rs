use std::{
	fs::{File, OpenOptions},
	io::{self},
	ops::{Deref, DerefMut},
	path::Path,
};

use byte_view::{ByteView, ViewBuf};
use thiserror::Error;

use crate::{
	consts::{
		validate_page_size, PageSizeBoundsError, DEFAULT_PAGE_SIZE, META_FORMAT_VERSION,
		META_MAGIC, PAGE_SIZE_RANGE,
	},
	io::IoTarget,
	utils::byte_order::ByteOrder,
};

#[derive(Debug, Error)]
pub(crate) enum LoadError {
	#[error(
		"The provided file is not a storage meta file (expected magic bytes {META_MAGIC:08x?})"
	)]
	NotAMetaFile,

	#[error("Meta format version {0} is not supported by this version of acorn")]
	UnsupportedVersion(u8),

	#[error("Cannot open a {0} storage on a {} device", ByteOrder::NATIVE)]
	ByteOrderMismatch(ByteOrder),

	#[error("Cannot open a storage file with invalid configured page size: {0}")]
	PageSizeBounds(#[from] PageSizeBoundsError),

	#[error("The storage metadata is corrupted")]
	Corrupted,

	#[error("An error occurred accessing the data directory meta file: {0}")]
	Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub(crate) enum InitError {
	#[error(transparent)]
	PageSizeBounds(#[from] PageSizeBoundsError),

	#[error(transparent)]
	Io(#[from] io::Error),
}

pub(crate) struct InitParams {
	pub page_size: u16,
}

impl Default for InitParams {
	fn default() -> Self {
		Self {
			page_size: DEFAULT_PAGE_SIZE,
		}
	}
}

/*
 * TODO: Maybe this should just mmap() the file?
 */

pub(super) struct StorageMetaBuf<F: IoTarget> {
	meta: ViewBuf<StorageMeta>,
	file: F,
}

impl StorageMetaBuf<File> {
	pub fn init_file(path: impl AsRef<Path>, params: InitParams) -> Result<(), InitError> {
		let mut file = OpenOptions::new()
			.write(true)
			.truncate(true)
			.create(true)
			.open(path)?;
		Self::init(&mut file, params)
	}

	pub fn load_file(path: impl AsRef<Path>) -> Result<Self, LoadError> {
		let file = OpenOptions::new().read(true).write(true).open(path)?;
		Self::load(file)
	}
}

impl<F: IoTarget> StorageMetaBuf<F> {
	pub fn load(file: F) -> Result<Self, LoadError> {
		let mut meta_data: ViewBuf<StorageMeta> = ViewBuf::new();
		if file.read_at(meta_data.as_bytes_mut(), 0)? != meta_data.size() {
			return Err(LoadError::NotAMetaFile);
		}
		let meta = Self {
			meta: meta_data,
			file,
		};
		if meta.magic != META_MAGIC {
			return Err(LoadError::NotAMetaFile);
		}
		if meta.format_version != META_FORMAT_VERSION {
			return Err(LoadError::UnsupportedVersion(meta.format_version));
		}
		let Some(byte_order) = ByteOrder::from_byte(meta.byte_order) else {
			return Err(LoadError::Corrupted);
		};
		if byte_order != ByteOrder::NATIVE {
			return Err(LoadError::ByteOrderMismatch(byte_order));
		}
		validate_page_size(meta.page_size())?;
		Ok(meta)
	}

	pub fn init(file: &mut F, params: InitParams) -> Result<(), InitError> {
		validate_page_size(params.page_size)?;
		let page_size_exponent = params.page_size.ilog2() as u8;

		let mut meta: ViewBuf<StorageMeta> = ViewBuf::new();
		*meta = StorageMeta {
			magic: META_MAGIC,
			format_version: META_FORMAT_VERSION,
			byte_order: ByteOrder::NATIVE as u8,
			page_size_exponent,
			segment_num_limit: 0,
		};

		file.set_len(0)?;
		file.write_at(meta.as_bytes(), 0)?;

		Ok(())
	}

	pub fn flush(&mut self) -> Result<(), io::Error> {
		self.file.set_len(0)?;
		self.file.write_at(self.meta.as_bytes(), 0)?;
		Ok(())
	}
}

impl<T: IoTarget> Deref for StorageMetaBuf<T> {
	type Target = StorageMeta;

	fn deref(&self) -> &Self::Target {
		&self.meta
	}
}

impl<T: IoTarget> DerefMut for StorageMetaBuf<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.meta
	}
}

#[derive(ByteView)]
#[repr(C)]
pub(super) struct StorageMeta {
	pub magic: [u8; 4],
	pub format_version: u8,
	pub byte_order: u8,
	pub page_size_exponent: u8,
	pub segment_num_limit: u32,
}

impl StorageMeta {
	#[inline]
	pub fn page_size(&self) -> u16 {
		1_u16
			.checked_shl(self.page_size_exponent.into())
			.unwrap_or(*PAGE_SIZE_RANGE.end())
	}
}

#[cfg(test)]
mod tests {
	use std::mem::size_of;

	use crate::utils::{aligned_buf::AlignedBuffer, units::KiB};

	use super::*;

	#[test]
	fn load() {
		let mut data = AlignedBuffer::with_capacity(8, size_of::<StorageMeta>());
		data[0..4].copy_from_slice(b"ACNM");
		data[4] = 1;
		data[5] = ByteOrder::NATIVE as u8;
		data[6] = 14;
		data[7] = 0;
		data[8..12].copy_from_slice(&420_u32.to_ne_bytes());

		let meta = StorageMetaBuf::load(data).unwrap();
		assert_eq!(meta.format_version, 1);
		assert_eq!(meta.byte_order, ByteOrder::NATIVE as u8);
		assert_eq!(meta.page_size_exponent, 14);
		assert_eq!(meta.page_size(), 16 * KiB as u16);
		assert_eq!(meta.segment_num_limit, 420);
	}

	#[test]
	fn load_with_too_large_page_size_exponent() {
		let mut data = AlignedBuffer::with_capacity(8, size_of::<StorageMeta>());
		data[0..4].copy_from_slice(b"ACNM");
		data[4] = 1;
		data[5] = ByteOrder::NATIVE as u8;
		data[6] = 69;
		data[7] = 0;
		data[8..12].copy_from_slice(&420_u32.to_ne_bytes());

		let meta = StorageMetaBuf::load(data).unwrap();
		assert_eq!(meta.page_size(), 32 * KiB as u16); // Should be the maximum
	}

	#[test]
	fn write_and_flush() {
		let mut data = AlignedBuffer::with_capacity(8, size_of::<StorageMeta>());
		data[0..4].copy_from_slice(b"ACNM");
		data[4] = 1;
		data[5] = ByteOrder::NATIVE as u8;
		data[6] = 14;
		data[7] = 0;
		data[8..12].copy_from_slice(&420_u32.to_ne_bytes());

		let mut meta = StorageMetaBuf::load(data).unwrap();
		meta.segment_num_limit = 69;

		assert_eq!(meta.file[8..12], 420_u32.to_ne_bytes());

		meta.flush().unwrap();

		assert_eq!(meta.file[8..12], 69_u32.to_ne_bytes());
	}
}
