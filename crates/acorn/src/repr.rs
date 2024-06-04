use std::{
	error::Error,
	io::{self, Read, Write},
	mem,
};

use zerocopy::{AsBytes, FromBytes};

pub(crate) trait Repr<T, E: Error>: Sized + FromBytes + AsBytes
where
	T: TryFrom<Self> + Into<Self>,
	E: From<T::Error> + From<io::Error>,
{
	const SIZE: usize = mem::size_of::<Self>();

	fn serialize(value: T, mut writer: impl Write) -> Result<(), E> {
		let repr: Self = value.into();
		writer.write_all(repr.as_bytes())?;
		Ok(())
	}

	fn deserialize(mut reader: impl Read) -> Result<T, E> {
		let mut repr = Self::new_zeroed();
		reader.read_exact(repr.as_bytes_mut())?;
		Ok(T::try_from(repr)?)
	}

	fn from_bytes(bytes: &[u8]) -> Result<T, E> {
		let mut repr = Self::new_zeroed();
		repr.as_bytes_mut().copy_from_slice(bytes);
		Ok(T::try_from(repr)?)
	}
}
