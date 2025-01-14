//! `select` waits for a file descriptor in the given sets to be readable,
//! writable or for an exception to occur.

use crate::errno::Errno;
use crate::process::mem_space::ptr::SyscallPtr;
use crate::process::mem_space::ptr::SyscallSlice;
use crate::process::scheduler;
use crate::process::Process;
use crate::time::clock;
use crate::time::clock::CLOCK_MONOTONIC;
use crate::time::unit::TimeUnit;
use crate::time::unit::Timeval;
use crate::util::io;
use crate::util::io::IO;
use core::cmp::min;
use core::ffi::c_int;
use core::ffi::c_long;
use macros::syscall;

/// The number of file descriptors in FDSet.
pub const FD_SETSIZE: usize = 1024;

/// Structure representing `fd_set`.
#[repr(C)]
#[derive(Debug)]
pub struct FDSet {
	/// The set's bitfield.
	fds_bits: [c_long; FD_SETSIZE / c_long::BITS as usize],
}

impl FDSet {
	/// Tells whether the given file descriptor `fd` is set in the list.
	pub fn is_set(&self, fd: u32) -> bool {
		if fd as usize >= FD_SETSIZE {
			return false;
		}

		// TODO Check correctness
		self.fds_bits[(fd as usize) / c_long::BITS as usize] >> (fd % (c_long::BITS as u32)) != 0
	}

	/// Sets the bit for file descriptor `fd`.
	pub fn set(&mut self, fd: u32) {
		// TODO Check correctness
		self.fds_bits[(fd as usize) / c_long::BITS as usize] |= 1 << (fd % (c_long::BITS as u32));
	}

	/// Clears the bit for file descriptor `fd`.
	pub fn clear(&mut self, fd: u32) {
		// TODO Check correctness
		self.fds_bits[(fd as usize) / c_long::BITS as usize] &=
			!(1 << (fd % (c_long::BITS as u32)));
	}
}

/// Performs the select operation.
///
/// Arguments:
/// - `nfds` is the number of the highest checked fd + 1.
/// - `readfds` is the bitfield of fds to check for read operations.
/// - `writefds` is the bitfield of fds to check for write operations.
/// - `exceptfds` is the bitfield of fds to check for exceptional conditions.
/// - `timeout` is the timeout after which the syscall returns.
/// - `sigmask` TODO
pub fn do_select<T: TimeUnit>(
	nfds: u32,
	readfds: SyscallPtr<FDSet>,
	writefds: SyscallPtr<FDSet>,
	exceptfds: SyscallPtr<FDSet>,
	timeout: SyscallPtr<T>,
	_sigmask: Option<SyscallSlice<u8>>,
) -> Result<i32, Errno> {
	// Getting start timestamp
	let start = clock::current_time_struct::<T>(CLOCK_MONOTONIC)?;

	// Getting timeout
	let timeout = {
		let proc_mutex = Process::current_assert();
		let proc = proc_mutex.lock();

		let mem_space = proc.get_mem_space().unwrap();
		let mem_space_guard = mem_space.lock();
		timeout.get(&mem_space_guard)?.cloned().unwrap_or_default()
	};

	// Tells whether the syscall immediately returns
	let polling = timeout.is_zero();
	// The end timestamp
	let end = start + timeout;

	loop {
		let mut events_count = 0;
		// Set if every bitfields are set to zero
		let mut all_zeros = true;

		for fd_id in 0..min(nfds, FD_SETSIZE as u32) {
			let (mem_space, fds_mutex) = {
				let proc_mutex = Process::current_assert();
				let proc = proc_mutex.lock();

				let mem_space = proc.get_mem_space().unwrap().clone();
				let fds_mutex = proc.get_fds().unwrap().clone();

				(mem_space, fds_mutex)
			};

			let (read, write, except) = {
				let mem_space_guard = mem_space.lock();

				let read = readfds
					.get(&mem_space_guard)?
					.map(|fds| fds.is_set(fd_id))
					.unwrap_or(false);
				let write = writefds
					.get(&mem_space_guard)?
					.map(|fds| fds.is_set(fd_id))
					.unwrap_or(false);
				let except = exceptfds
					.get(&mem_space_guard)?
					.map(|fds| fds.is_set(fd_id))
					.unwrap_or(false);

				(read, write, except)
			};

			if read || write || except {
				all_zeros = false;
			}

			let fds = fds_mutex.lock();
			let fd = fds.get_fd(fd_id);

			// Checking the file descriptor exists
			let fd = match fd {
				Some(fd) => fd,

				None => {
					if read || write || except {
						return Err(errno!(EBADF));
					}

					continue;
				}
			};

			// Building event mask
			let mut mask = 0;
			if read {
				mask |= io::POLLIN;
			}
			if write {
				mask |= io::POLLOUT;
			}
			if except {
				mask |= io::POLLPRI;
			}

			let open_file_mutex = fd.get_open_file();
			let mut open_file = open_file_mutex.lock();

			let result = open_file.poll(mask)?;

			// Setting results
			let mut mem_space_guard = mem_space.lock();
			if read && result & io::POLLIN != 0 {
				readfds
					.get_mut(&mut mem_space_guard)?
					.map(|fds| fds.set(fd_id));
				events_count += 1;
			} else {
				readfds
					.get_mut(&mut mem_space_guard)?
					.map(|fds| fds.clear(fd_id));
			}
			if write && result & io::POLLOUT != 0 {
				writefds
					.get_mut(&mut mem_space_guard)?
					.map(|fds| fds.set(fd_id));
				events_count += 1;
			} else {
				writefds
					.get_mut(&mut mem_space_guard)?
					.map(|fds| fds.clear(fd_id));
			}
			if except && result & io::POLLPRI != 0 {
				exceptfds
					.get_mut(&mut mem_space_guard)?
					.map(|fds| fds.set(fd_id));
				events_count += 1;
			} else {
				exceptfds
					.get_mut(&mut mem_space_guard)?
					.map(|fds| fds.clear(fd_id));
			}
		}

		// If one or more events occured, return
		if all_zeros || polling || events_count > 0 {
			return Ok(events_count);
		}

		let curr = clock::current_time_struct::<T>(CLOCK_MONOTONIC)?;
		// On timeout, return 0
		if curr >= end {
			return Ok(0);
		}

		// TODO Make the process sleep?
		scheduler::end_tick();
	}
}

#[syscall]
pub fn select(
	nfds: c_int,
	readfds: SyscallPtr<FDSet>,
	writefds: SyscallPtr<FDSet>,
	exceptfds: SyscallPtr<FDSet>,
	timeout: SyscallPtr<Timeval>,
) -> Result<i32, Errno> {
	do_select(nfds as _, readfds, writefds, exceptfds, timeout, None)
}
