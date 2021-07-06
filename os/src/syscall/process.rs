use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    current_mmap,
    current_unmmap,
};
use crate::timer::get_time_ms;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_mmap(start : *const u8, mut len : usize, port : usize) -> isize {
    if len == 0 {return 0}
    if port & !0x7 != 0 || port & 0x7 == 0 {
        println!("Illegal port.");
        return -1;
    }
    if (start as usize) & (PAGE_SIZE - 1) != 0 {
        println!("Start address {:?} is not page-aligned.", start);
        return -1;
    }

    if len % PAGE_SIZE != 0 {
        len = (len / PAGE_SIZE + 1) * PAGE_SIZE
    }
    current_mmap(start, len, port)
}

pub fn sys_munmap(start : *const u8, mut len : usize) -> isize {
    if (start as usize) & (PAGE_SIZE - 1) != 0 {
        println!("Start address {:?} is not page-aligned.", start);
        return -1;
    }

    if len % PAGE_SIZE != 0 {
        len = (len / PAGE_SIZE + 1) * PAGE_SIZE
    }
    current_unmmap(start, len)
}