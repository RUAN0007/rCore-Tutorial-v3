const FD_STDOUT: usize = 1;
const USER_STACK_SIZE : usize = 0x1000;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;
pub fn sys_write(fd: usize, buf: *const u8, len: usize, cur_user_sp: *const u8) -> isize {
    let top : usize = (cur_user_sp  as usize + USER_STACK_SIZE - 1) & (!(USER_STACK_SIZE - 1));
    let bottom : usize = top as usize - USER_STACK_SIZE;
    let mut in_stack : bool = false;
    if bottom < buf as usize && buf as usize <= top {
        in_stack = true;
    }

    let mut in_space : bool = false;
    if APP_BASE_ADDRESS <= (buf as usize) && (buf as usize) < APP_BASE_ADDRESS + APP_SIZE_LIMIT {
        in_space = true;
    }

    if !in_space && !in_stack {
        return -1isize;
    }


    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}