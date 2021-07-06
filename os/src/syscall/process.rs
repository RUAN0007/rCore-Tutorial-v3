use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    set_current_priority,
};
use crate::timer::get_time_us;
use crate::timer::USEC_PER_SEC;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(timer_val_ptr : *const u8) -> isize {
    let now_us : usize = get_time_us();
    let now_s = now_us / USEC_PER_SEC;
    let now_only_us = now_us % USEC_PER_SEC;
    unsafe{
        *(timer_val_ptr as *mut usize) = now_s;
        *((timer_val_ptr as usize + core::mem::size_of::<usize>()) as *mut usize) = now_only_us;
    }
    // println!("sys_get_time: now_s={}, now_only_us={}", now_s, now_only_us);
    0
}

pub fn sys_set_priority(prio: isize) -> isize {
    if prio < 2 {return -1}
    set_current_priority(prio as usize);
    prio
}