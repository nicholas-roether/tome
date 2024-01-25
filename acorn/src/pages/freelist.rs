use std::num::NonZeroU16;

use byte_view::ByteView;

#[derive(Debug, ByteView)]
#[dynamically_sized]
#[repr(C)]
pub(crate) struct FreelistPage {
	pub next: Option<NonZeroU16>,
	pub length: u16,
	pub items: [Option<NonZeroU16>],
}
