use js_sys::Uint8Array;

pub trait AsUint8Array {
	unsafe fn as_uint8array(&self) -> Uint8Array;
}

impl<T> AsUint8Array for [T] {
	unsafe fn as_uint8array(&self) -> Uint8Array {
		Uint8Array::view_mut_raw(
			std::mem::transmute::<*const T, *mut u8>(self.as_ptr()),
			std::mem::size_of::<T>() * self.len(),
		)
	}
}

impl<T> AsUint8Array for Vec<T> {
	unsafe fn as_uint8array(&self) -> Uint8Array {
		Uint8Array::view_mut_raw(
			std::mem::transmute::<*const T, *mut u8>(self.as_ptr()),
			std::mem::size_of::<T>() * self.len(),
		)
	}
}
