// SPDX-License-Identifier: 0BSD
use crate::{byond, ByondResult, ByondValue};
use std::{cell::RefCell, ffi::CStr, mem::MaybeUninit};

const DEFAULT_BUFFER_CAPACITY: usize = 1024;

impl ByondValue {
	#[inline]
	pub fn new_string<Str>(string: Str) -> Self
	where
		Str: Into<String>,
	{
		let mut string = string.into().into_bytes();
		string.push(0);
		unsafe {
			let mut value = MaybeUninit::uninit();
			byond().ByondValue_SetStr(value.as_mut_ptr(), string.as_ptr().cast());
			Self(value.assume_init())
		}
	}

	pub fn set_string<Str>(&mut self, string: Str)
	where
		Str: Into<String>,
	{
		let mut string = string.into().into_bytes();
		string.push(0);
		unsafe { byond().ByondValue_SetStr(&mut self.0, string.as_ptr().cast()) }
	}

	pub fn get_string(&self) -> ByondResult<String> {
		thread_local! {
			static STRING_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(DEFAULT_BUFFER_CAPACITY));
		}
		STRING_BUFFER.with_borrow_mut(|buffer| unsafe {
			let mut needed_len = buffer.capacity();
			if byond().Byond_ToString(&self.0, buffer.as_mut_ptr().cast(), &mut needed_len) {
				// Safety: if this returns true, then the buffer was large enough, and thus
				// needed_len <= capacity.
				buffer.set_len(needed_len);
				return buffer_to_string(buffer);
			}
			buffer.reserve(needed_len.saturating_sub(buffer.len()));
			map_byond_error!(byond().Byond_ToString(
				&self.0,
				buffer.as_mut_ptr().cast(),
				&mut needed_len
			))?;
			// Safety: needed_len is always <= capacity here,
			// unless BYOND did a really bad fucky wucky.
			buffer.set_len(needed_len);
			buffer_to_string(buffer)
		})
	}
}

#[inline]
fn buffer_to_string(buffer: &[u8]) -> ByondResult<String> {
	let cstr = CStr::from_bytes_with_nul(buffer)?;
	Ok(cstr.to_str().map(str::to_owned)?)
}
