use crate::mm::{UserBuffer, translated_byte_buffer, may_translated_byte_buffer, translated_refmut};
use crate::task::{current_user_token, current_task, find_task};

use crate::fs::{make_pipe};
use crate::config::MAIL_FD;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_mailread(buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();

    // println!("MAIL READ: cur_pid={}, len={}", task.getpid(), len);
    let mails = inner.fd_table[MAIL_FD].as_ref().unwrap().clone();
    drop(inner);
    if let Some(addrs) = may_translated_byte_buffer(token, buf, len) {
        let mails_return = mails.read( UserBuffer::new(addrs));

        if len == 0 { // special meaning to test whether a read-available mail
            if mails_return == 0 {-1} else {0}
        } else {
            if mails_return == 0 {-1} else {mails_return as isize}
        }
    } else {
        // invalid address
        -1
    }
}

pub fn sys_mailwrite(pid: usize, buf: *const u8, len: usize) -> isize {
    let cur_token = current_user_token();
    let mut task = current_task().unwrap();
    // println!("MAIL RIGHT: pid={}, cur_pid={}, len={}", pid, task.getpid(), len);
    if pid == task.getpid() {
        // do nothing
    } else if let Some(idle_task) = find_task(pid) {
        task = idle_task;
    } else {
        return -1;
    }
    let inner = task.acquire_inner_lock();

    let mails = inner.fd_table[MAIL_FD].as_ref().unwrap().clone();
    drop(inner);
    if let Some(physical_addrs) = may_translated_byte_buffer(cur_token, buf, len) {
        let mails_return = mails.write( UserBuffer::new(physical_addrs));

        if len == 0 { // special meaning to test whether a write-available mail
            if mails_return == 0 {-1} else {0}
        } else {
            if mails_return == 0 { -1 } else { mails_return as isize }
        }
    } else {
        // invalid address
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}