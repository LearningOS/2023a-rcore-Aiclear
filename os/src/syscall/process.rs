//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, translate_vpn_to_ppn, get_current_syscall_times, get_current_task_scheduling_time, add_new_area, check_vpn_range_arealdy_mapped, remove_map_area,
    }, timer::get_time_us,
    mm::{VirtAddr, MapPermission}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

    // virt_addr -> phy_addr
    let start_va = VirtAddr::from(ts as usize);
    let ppn = translate_vpn_to_ppn(start_va.floor());

    let us = get_time_us();
    unsafe {
        let t_ts = ((ppn.get_mut::<u8>() as *mut u8) as usize + start_va.page_offset()) as *mut TimeVal;
        *t_ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    // vpn -> ppn
    let start_va = VirtAddr::from(ti as usize);
    let ppn = translate_vpn_to_ppn(start_va.floor());

    unsafe {
        let t_ti = (ppn.get_mut::<u8>() as *mut u8 as usize + start_va.page_offset()) as *mut TaskInfo;
        *t_ti = TaskInfo {
            status: TaskStatus::Running,
            syscall_times: get_current_syscall_times(),
            time: (get_time_us() - get_current_task_scheduling_time()) / 1_000,
        }
    }

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");

    // check port legal
    if port & 0xF8 != 0 || port & 0x07 == 0 {
        return -1;
    }

    // address range
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    // check start address align 4KB
    if !start_va.aligned() {
        return -1;
    }

    // check map range is already have mapped address
    if check_vpn_range_arealdy_mapped(start_va, end_va) {
        return -1;
    }

    // permission flag
    let mut permission = MapPermission::from_bits_truncate((port << 1) as u8);
    // user mode access
    permission.toggle(MapPermission::U);

    // add new map area to current task
    add_new_area(start_va, end_va, permission);

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    // address range
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    // check start align 4KB
    if !start_va.aligned() {
        return -1;
    }

    // unmap
    remove_map_area(start_va, end_va)
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
