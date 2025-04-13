//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};
use crate::task::TASK_MANAGER;
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let value = unsafe { core::ptr::read(id as *const u8) };
            value as isize
        },
        1 => {
            let byte_to_write = data as u8;
            unsafe {
                core::ptr::write(id as *mut u8, byte_to_write);
            }
            0
        },
        2 => {
            let inner = TASK_MANAGER.inner.exclusive_access();
            let current_task_id = inner.current_task;
            let count = inner.tasks[current_task_id].task_cx.get_syscall_count(id);
            drop(inner);
            count as isize
        }
        _ => -1,
    }
}
