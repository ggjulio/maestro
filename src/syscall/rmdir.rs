//! The `rmdir` system call deletes the given directory from its filesystem. If
//! no link remain to the inode, the function also removes the inode.

use crate::errno::Errno;
use crate::file::path::Path;
use crate::file::vfs;
use crate::file::FileContent;
use crate::process::mem_space::ptr::SyscallString;
use crate::process::Process;
use macros::syscall;

#[syscall]
pub fn rmdir(pathname: SyscallString) -> Result<i32, Errno> {
	let (path, uid, gid) = {
		// Getting the process
		let mutex = Process::get_current().unwrap();
		let guard = mutex.lock();
		let proc = guard.get_mut();

		let mem_space = proc.get_mem_space().unwrap();
		let mem_space_guard = mem_space.lock();

		let path = Path::from_str(pathname.get(&mem_space_guard)?.ok_or(errno!(EFAULT))?, true)?;
		let path = super::util::get_absolute_path(proc, path)?;

		(path, proc.get_euid(), proc.get_egid())
	};

	// Removing the directory
	{
		let mutex = vfs::get();
		let guard = mutex.lock();
		let vfs = guard.get_mut().as_mut().unwrap();

		// Getting directory
		let file_mutex = vfs.get_file_from_path(&path, uid, gid, true)?;
		let file_guard = file_mutex.lock();
		let file = file_guard.get_mut();

		match file.get_content() {
			FileContent::Directory(entries) if entries.len() > 2 => return Err(errno!(ENOTEMPTY)),
			FileContent::Directory(_) => {}

			_ => return Err(errno!(ENOTDIR)),
		}

		vfs.remove_file(file, uid, gid)?;
	}

	Ok(0)
}
