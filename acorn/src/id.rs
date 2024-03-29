use core::fmt;

use byte_view::ByteView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ByteView)]
pub(crate) struct PageId {
	pub segment_num: u32,
	pub page_num: u16,
	pub _placeholder: u16,
}

impl PageId {
	#[inline]
	pub fn new(segment_num: u32, page_num: u16) -> Self {
		Self {
			segment_num,
			page_num,
			_placeholder: 0,
		}
	}
}

impl fmt::Display for PageId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:08x}:{:04x}", self.segment_num, self.page_num)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ByteView)]
pub(crate) struct ItemId {
	pub segment_num: u32,
	pub page_num: u16,
	pub index: u16,
}

impl ItemId {
	#[inline]
	pub fn new(page_id: PageId, index: u16) -> Self {
		Self {
			segment_num: page_id.segment_num,
			page_num: page_id.page_num,
			index,
		}
	}

	#[inline]
	pub fn page_id(self) -> PageId {
		PageId::new(self.segment_num, self.page_num)
	}
}

impl fmt::Display for ItemId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{:04x}", self.page_id(), self.index)
	}
}
