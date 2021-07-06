use super::File;
use super::pipe::PipeRingBuffer;
use crate::mm::{
    UserBuffer,
};
use spin::Mutex;
use alloc::vec::Vec;
const MAX_MAIL_COUNT : usize = 16;

pub struct Mails {
    inner : Mutex<MailsInner>,
}

struct MailsInner{
    mails: Vec<PipeRingBuffer>,
    prev_write_mail_idx : usize,
    prev_read_mail_idx : usize,
}

impl Mails {
    pub fn new() -> Self {
        let mut inner = MailsInner{
            mails : Vec::new(),
            prev_write_mail_idx: 0,
            prev_read_mail_idx: 0,
        };
        for _ in 0..MAX_MAIL_COUNT {
            inner.mails.push(PipeRingBuffer::new());
        }
        Self{inner: Mutex::new(inner)}
    }

}

impl File for Mails {

    // 0 implies not available mail to read.
    fn read(&self, buf: UserBuffer) -> usize {

        let mut inner = self.inner.lock();

        let next_mail_idx : usize = (inner.prev_read_mail_idx + 1) % MAX_MAIL_COUNT;
        let next_avail_read : usize = inner.mails[next_mail_idx].available_read();

        // println!("\tBefore mail read: prev_read_mail_idx={}, prev_write_mail_idx={}, next_avail_read={}, buf.len={}", inner.prev_read_mail_idx, inner.prev_write_mail_idx, next_avail_read, buf.len());

        if buf.len() == 0 {
            return if next_avail_read == 0 {0} else {1}
        }

        if next_avail_read > 0 {
            let mut read_size : usize = 0;
            let mut buf_iter = buf.into_iter();
            for i in 0..next_avail_read {
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe { *byte_ref = inner.mails[next_mail_idx].read_byte(); }
                    read_size += 1;
                } else {
                    // Drain the mail to empty it. 
                    inner.mails[next_mail_idx].read_byte();
                }
            }
            assert_eq!(inner.mails[next_mail_idx].available_read(), 0);
            inner.prev_read_mail_idx = next_mail_idx;
            read_size
        } else {
            0
        }

    }

    // 0 implies not available mail to read.
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        // println!("\tBefore mail write: prev_read_mail_idx={}, prev_write_mail_idx={}", inner.prev_read_mail_idx, inner.prev_write_mail_idx);

        let next_mail_idx : usize = (inner.prev_write_mail_idx + 1) % MAX_MAIL_COUNT;
        let next_avail_read = inner.mails[next_mail_idx].available_read();
        if buf.len() == 0 {
            return if next_avail_read != 0 {0} else {1}
        }
        if next_avail_read == 0 { // empty, available to write
            let mut write_size : usize = 0;
            let mut buf_iter = buf.into_iter();
            let avail_write : usize = inner.mails[next_mail_idx].available_write();
            for i in 0..avail_write {
                if let Some(byte_ref) = buf_iter.next() {
                    inner.mails[next_mail_idx].write_byte(unsafe { *byte_ref });
                    write_size += 1;
                } else {
                    break
                }
            }
            assert_eq!(inner.mails[next_mail_idx].available_read(), write_size);
            inner.prev_write_mail_idx = next_mail_idx;
            write_size
        } else {
            0
        }
    }
}