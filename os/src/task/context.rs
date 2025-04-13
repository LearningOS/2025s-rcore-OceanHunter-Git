//! Implementation of [`TaskContext`]

#[derive(Copy, Clone)]
#[repr(C)]
/// task context structure containing some registers
pub struct TaskContext {
    /// Ret position after task switching
    ra: usize,
    /// Stack pointer
    sp: usize,
    /// s0-11 register, callee saved
    s: [usize; 12],
    /// syscall_counts
    pub syscall_counts: [usize; 420],
}

impl TaskContext {
    /// Create a new empty task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
            syscall_counts: [0; 420],
        }
    }
    /// Create a new task context with a trap return addr and a kernel stack pointer
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
            syscall_counts: [0; 420],
        }
    }

    /// increment_syscall_count
    pub fn increment_syscall_count(&mut self, syscall_id: usize) {
        if syscall_id < self.syscall_counts.len() {
            self.syscall_counts[syscall_id] += 1;
        }
    }

    /// get_syscall_count
    pub fn get_syscall_count(&self, syscall_id: usize) -> usize {
        if syscall_id < self.syscall_counts.len() {
            self.syscall_counts[syscall_id]
        } else {
            0
        }
    }
}
