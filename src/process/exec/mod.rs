//! Program execution routines.
//!
//! Program execution is done in several stages:
//! - Read the program
//! - Parse the program
//! - Build the memory image according to the program
//! - Replace the process's memory with the newly created image to run it

pub mod elf;
pub mod vdso;

use crate::errno::EResult;
use crate::errno::Errno;
use crate::file::perm::AccessProfile;
use crate::file::File;
use crate::process::mem_space::MemSpace;
use crate::process::regs::Regs;
use crate::process::signal::SignalHandler;
use crate::process::Process;
use crate::util::container::string::String;
use crate::util::container::vec::Vec;
use crate::util::lock::IntMutex;
use crate::util::lock::Mutex;
use crate::util::ptr::arc::Arc;
use core::ffi::c_void;

/// Informations to prepare a program image to be executed.
pub struct ExecInfo {
	/// The access profile of the calling agent.
	pub access_profile: AccessProfile,
	/// The list of arguments.
	pub argv: Vec<String>,
	/// The list of environment variables.
	pub envp: Vec<String>,
}

/// A built program image.
pub struct ProgramImage {
	/// The argv of the program.
	argv: Vec<String>,

	/// The image's memory space.
	mem_space: MemSpace,

	/// A pointer to the entry point of the program.
	entry_point: *const c_void,

	/// A pointer to the process's user stack.
	user_stack: *mut c_void,
	/// A pointer to the initial value of the user stack pointer.
	user_stack_begin: *mut c_void,

	/// A pointer to the process's kernel stack.
	kernel_stack: *mut c_void,
}

/// A program executor, whose role is to load a program and to preprare it for execution.
pub trait Executor {
	/// Builds a program image.
	/// `file` is the program's file.
	fn build_image(&self, file: &mut File) -> Result<ProgramImage, Errno>;
}

/// Builds a program image from the given executable file.
///
/// Arguments:
/// - `file` is the program's file
/// - `info` is the set execution informations for the program
///
/// The function returns a memory space containing the program image and the
/// pointer to the entry point.
pub fn build_image(file: &mut File, info: ExecInfo) -> EResult<ProgramImage> {
	// TODO Support other formats than ELF (wasm?)

	let exec = elf::ELFExecutor::new(info)?;
	exec.build_image(file)
}

/// Executes the program image `image` on the process `proc`.
pub fn exec(proc: &mut Process, image: ProgramImage) -> EResult<()> {
	proc.argv = Arc::new(image.argv)?;
	// TODO Set exec path

	// Duplicate the file descriptor table
	let fds = proc
		.get_fds()
		.map(|fds_mutex| -> EResult<_> {
			let fds = fds_mutex.lock();
			let new_fds = fds.duplicate(true)?;
			Ok(Arc::new(Mutex::new(new_fds))?)
		})
		.transpose()?;

	// Set the new memory space to the process
	proc.set_mem_space(Some(Arc::new(IntMutex::new(image.mem_space))?));

	// Set new file descriptor table
	proc.set_fds(fds);

	// Set the process's stacks
	proc.user_stack = Some(image.user_stack);
	proc.kernel_stack = Some(image.kernel_stack);
	proc.update_tss();

	// Reset signals
	proc.sigmask.clear_all();
	{
		let mut handlers = proc.signal_handlers.lock();
		for i in 0..handlers.len() {
			handlers[i] = SignalHandler::Default;
		}
	}

	proc.reset_vfork();
	proc.clear_tls_entries();

	// Set the process's registers
	let regs = Regs {
		esp: image.user_stack_begin as _,
		eip: image.entry_point as _,
		..Default::default()
	};
	proc.regs = regs;

	Ok(())
}
