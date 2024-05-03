use std::{
	io::{Read, Write},
	mem::size_of,
};

use crc::Crc;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use super::FileError;

// TODO: there are tradeoffs here. Perhaps I should look more into selecting an
// algorithm.
pub(crate) const CRC32: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

pub(crate) trait Serialized: Sized
where
	FileError: From<<Self::Repr as TryInto<Self>>::Error>,
{
	type Repr: AsBytes + FromBytes + FromZeroes + From<Self> + TryInto<Self>;

	const REPR_SIZE: usize = size_of::<Self::Repr>();

	fn serialize(self, mut writer: impl Write) -> Result<(), FileError> {
		let repr = Self::Repr::from(self);
		writer.write_all(repr.as_bytes())?;
		Ok(())
	}

	fn deserialize(mut reader: impl Read) -> Result<Self, FileError> {
		let mut repr = Self::Repr::new_zeroed();
		reader.read_exact(repr.as_bytes_mut())?;
		let value: Self = repr.try_into()?;
		Ok(value)
	}
}