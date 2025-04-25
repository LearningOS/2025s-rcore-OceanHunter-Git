//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, current_user_token, TASK_MANAGER};
use crate::timer::{get_time_us};
use crate::mm::PageTable;
use crate::mm::page_table::{translated_byte_buffer};
use crate::config::PAGE_SIZE;
use crate::mm::memory_set::{MapArea, MapType};
use crate::mm::MapPermission;
use core::mem::size_of;
use crate::mm::VirtAddr;
use crate::mm::VPNRange; 
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let token = current_user_token();
    let time_us = get_time_us();
    let time_val = TimeVal{
        sec: time_us / 1_000_000,
        usec: time_us % 1_000_000,
    };
    let buffers = translated_byte_buffer(token, ts as *const u8, size_of::<TimeVal>());
    let mut data = [0u8; size_of::<TimeVal>()];
    unsafe {
        core::ptr::write(data.as_mut_ptr() as *mut TimeVal, time_val);
    }
    let mut offset = 0;
    for buffer in buffers {
        let len = buffer.len();
        buffer.copy_from_slice(&data[offset..offset + len]);
        offset += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            if id > 2 << 40 - 1{
                return -1;
            }
            let token = current_user_token();
            let page_table = PageTable::from_token(token);
            let start_va = VirtAddr::from(id);
            let vpn = start_va.floor();
            if let Some(pte) = page_table.translate(vpn) {
                if!pte.readable() {
                    return -1;
                }
            } else {
                return -1;
            }
            let buffers = translated_byte_buffer(token, id as *const u8, 1);
            if buffers.is_empty() {
                return -1;
            }
            let result = buffers[0][0];
            result as isize
        },
        1 => {
            if id > 2 << 40 - 1{
                return -1;
            }
            let token = current_user_token();
            let page_table = PageTable::from_token(token);
            let start_va = VirtAddr::from(id);
            let vpn = start_va.floor();
            if let Some(pte) = page_table.translate(vpn) {
                if!pte.writable() {
                    return -1;
                }
            } else {
                return -1;
            }
            let mut buffers = translated_byte_buffer(token, id as *mut u8, 1);
            if buffers.is_empty() {
                return -1;
            }
            let byte_to_write = data as u8;
            buffers[0][0] = byte_to_write;
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

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if port & 0b111 != port || port == 0{
        return -1;
    }

    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let end = start + page_count * PAGE_SIZE;

    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current_task_id = inner.current_task;
    let memory_set = &mut inner.tasks[current_task_id].memory_set;

    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(end).ceil();
    for area in &memory_set.areas {
        let area_start_vpn = area.vpn_range.get_start();
        let area_end_vpn = area.vpn_range.get_end();
        if !(end_vpn <= area_start_vpn || start_vpn >= area_end_vpn) {
            // overlap error
            drop(inner);
            return -1;
        }
    }

    // add MapPermission
    let mut map_perm = MapPermission::empty();
    if port & 0b001 != 0 {
        map_perm |= MapPermission::R;
    }
    if port & 0b010 != 0 {
        map_perm |= MapPermission::W;
    }
    if port & 0b100 != 0 {
        map_perm |= MapPermission::X;
    }
    map_perm |= MapPermission::U;

    // create a new area
    let mut map_area = MapArea::new(
        VirtAddr::from(start),
        VirtAddr::from(end),
        MapType::Framed,
        map_perm,
    );

    // map
    for vpn in map_area.vpn_range {
        map_area.map_one(&mut memory_set.page_table, vpn);
    }

    memory_set.areas.push(map_area);
    drop(inner);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let end = start + page_count * PAGE_SIZE;

    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current_task_id = inner.current_task;
    let memory_set = &mut inner.tasks[current_task_id].memory_set;

    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(end).ceil();
    let mut found = false;
    memory_set.areas.retain_mut(|area| {
        let area_start_vpn = area.vpn_range.get_start();
        let area_end_vpn = area.vpn_range.get_end();
        if area_start_vpn <= start_vpn && area_end_vpn >= end_vpn {
            // unmap
            for vpn in VPNRange::new(start_vpn, end_vpn) {
                area.unmap_one(&mut memory_set.page_table, vpn);
            }
            found = true;
            if area_start_vpn == start_vpn && area_end_vpn == end_vpn {
                // erase areas
                false
            } else {
                // update the VPNRange
                if area_start_vpn == start_vpn {
                    area.vpn_range = VPNRange::new(end_vpn, area_end_vpn);
                } else if area_end_vpn == end_vpn {
                    area.vpn_range = VPNRange::new(area_start_vpn, start_vpn);
                }
                true
            }
        } else {
            true
        }
    });

    drop(inner);
    if found {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
