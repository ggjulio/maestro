/// TODO doc

use crate::memory::malloc;
use crate::util::bit_size_of;
use crate::util::ceil_division;

/// TODO doc
pub struct Bitfield {
	/// The bitfield's memory region.
	ptr: *mut u8,
	/// The number of bits in the bitfield.
	len: usize,
	/// The number of set bits.
	set_count: usize,
}

impl Bitfield {
	/// Creates a new bitfield with the given number of bits `len`.
	pub fn new(len: usize) -> Result::<Self, ()> {
		let size = ceil_division(len, bit_size_of::<u8>());
		Ok(Self {
			ptr: malloc::alloc(size)? as *mut _,
			len: len,
			set_count: 0,
		})
	}

	/// Returns the number of bit in the bitfield.
	pub fn len(&self) -> usize {
		self.len
	}

	/// Returns the size of the memory region of the bitfield in bytes.
	pub fn mem_size(&self) -> usize {
		ceil_division(self.len, bit_size_of::<u8>())
	}

	/// Returns the number of set bits.
	pub fn set_count(&self) -> usize {
		self.set_count
	}

	/// Tells whether bit `index` is set.
	pub fn is_set(&self, index: usize) -> bool {
		let unit = unsafe { // Pointer arithmetic and dereference of raw pointer
			*self.ptr.offset((index / bit_size_of::<u8>()) as _)
		};
		(unit >> (index % bit_size_of::<u8>())) & 1 == 1
	}

	/// Sets bit `index`.
	pub fn set(&mut self, index: usize) {
		// TODO Check that index is in bound

		let unit = unsafe { // Pointer arithmetic and dereference of raw pointer
			&mut *self.ptr.offset((index / bit_size_of::<u8>()) as _)
		};
		*unit |= 1 << (index % bit_size_of::<u8>());

		self.set_count += 1;
	}

	/// Clears bit `index`.
	pub fn clear(&mut self, index: usize) {
		// TODO Check that index is in bound

		let unit = unsafe { // Pointer arithmetic and dereference of raw pointer
			&mut *self.ptr.offset((index / bit_size_of::<u8>()) as _)
		};
		*unit &= !(1 << (index % bit_size_of::<u8>()));

		self.set_count -= 1;
	}

	// TODO set_all
	// TODO clear_all
	// TODO fill
}

impl Drop for Bitfield {
	fn drop(&mut self) {
		malloc::free(self.ptr as *mut _);
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test_case]
	fn bitfield_set0() {
		let mut bitfield = Bitfield::new(42).unwrap();
		debug_assert_eq!(bitfield.len(), 42);

		for i in 0..bitfield.len() {
			debug_assert!(!bitfield.is_set(i));
		}

		for i in 0..bitfield.len() {
			bitfield.set(i);
		}

		for i in 0..bitfield.len() {
			debug_assert!(bitfield.is_set(i));
		}
	}

	#[test_case]
	fn bitfield_clear0() {
		let mut bitfield = Bitfield::new(42).unwrap();
		debug_assert_eq!(bitfield.len(), 42);

		for i in 0..bitfield.len() {
			bitfield.set(i);
		}

		for i in 0..bitfield.len() {
			debug_assert!(bitfield.is_set(i));
		}

		for i in 0..bitfield.len() {
			bitfield.clear(i);
		}

		for i in 0..bitfield.len() {
			debug_assert!(!bitfield.is_set(i));
		}
	}

	// TODO Write more tests
}