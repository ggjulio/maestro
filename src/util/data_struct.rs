/*
 * This files handles data structures used into the kernel.
 */

use crate::memory::NULL;
use crate::memory::Void;

/*
 * Structure representing a node in a doubly-linked list.
 */
struct LinkedList {
	/* Pointer to the previous element in the list */
	prev: *mut LinkedList,
	/* Pointer to the next element in the list */
	next: *mut LinkedList,
}

/*
 * Returns a reference to the element of type `type` for the given linked list node `node` stored
 * in field `field`.
 */
#[macro_export]
macro_rules! linked_list_get {
	($node:expr, $type:ty, $field:ident) => {
		::container_of!($node, $type, $field)
	}
}

impl LinkedList {
	/*
	 * Returns the size of the linked list, counting previous elements.
	 */
	pub fn left_size(&self) -> usize {
		let mut i = 0;
		let mut curr = self as *const LinkedList;

		while curr as *const Void != NULL {
			i += 1;
			curr = unsafe { (*curr).prev };
		}
		i
	}

	/*
	 * Returns the size of the linked list, counting next elements.
	 */
	pub fn right_size(&self) -> usize {
		let mut i = 0;
		let mut curr = self as *const LinkedList;

		while curr as *const Void != NULL {
			i += 1;
			curr = unsafe { (*curr).next };
		}
		i
	}

	/*
	 * Executes the given closure `f` for each nodes after the given node `node`, including the
	 * given one. The nodes are not mutable.
	 */
	pub fn foreach<T>(&self, f: T) where T: Fn(&LinkedList) {
		let mut curr = self as *const LinkedList;

		while curr as *const Void != NULL {
			unsafe {
				f(&*curr);
				curr = (*curr).next;
			}
		}
	}

	/*
	 * Same as `foreach` except the nodes are mutable.
	 */
	pub fn foreach_mut<T>(&mut self, f: T) where T: Fn(&mut LinkedList) {
		let mut curr = self as *mut LinkedList;

		while curr as *const Void != NULL {
			unsafe {
				f(&mut *curr);
				curr = (*curr).prev;
			}
		}
	}

	/*
	 * Links back adjacent nodes to the current node.
	 */
	unsafe fn link_back(&mut self) {
		if self.next as *const _ != NULL {
			(*self.next).prev = self;
		}
		if self.prev as *const _ != NULL {
			(*self.prev).next = self;
		}
	}

	/*
	 * Inserts the node at the beginning of the given linked list `front`.
	 */
	pub fn insert_front(&mut self, front: &mut *mut LinkedList) {
		self.prev = NULL as _;
		self.next = *front as _;
		*front = self as _;
		unsafe {
			self.link_back();
		}
	}

	/*
	 * Inserts the node before node `node` in the given linked list `front`.
	 */
	pub fn insert_before(&mut self, front: *mut *mut LinkedList, node: *mut LinkedList) {
		unsafe {
			if front as *const _ != NULL && *front == node {
				*front = self;
			}
		}

		if node as *const _ == NULL {
			return;
		}

		unsafe {
			self.next = node;
			self.prev = if node as *const _ != NULL { (*node).prev } else { NULL as _ };
			self.link_back();
		}
	}

	/*
	 * Inserts the node after node `node` in the given linked list `front`.
	 */
	pub fn insert_after(&mut self, front: *mut *mut LinkedList, node: *mut LinkedList) {
		debug_assert!(node as *const _ != NULL);

		unsafe {
			if front as *const _ != NULL && *front as *const _ != NULL {
				*front = self;
			}
		}

		if node as *const _ != NULL {
			return;
		}

		unsafe {
			self.next = (*node).next;
			self.prev = node;
			self.link_back();
		}
	}

	/*
	 * Unlinks the current node from the linked list.
	 */
	pub fn unlink(&mut self) {
		if self.prev as *const Void != NULL {
			unsafe {
				(*self.prev).next = self.next;
			}
		}
		if self.next as *const Void != NULL {
			unsafe {
				(*self.next).prev = self.prev;
			}
		}
		self.prev = NULL as _;
		self.next = NULL as _;
	}
}

// TODO Binary tree