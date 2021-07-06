mod context;
mod switch;
mod task;

use crate::config::MAX_APP_NUM;
use crate::config::BIG_STRIDE;
use crate::loader::{get_num_app, init_app_cx};
use core::cell::RefCell;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use heapless::binary_heap::{BinaryHeap, Min};

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
    task_heap: BinaryHeap<(Pass, usize), Min, MAX_APP_NUM>,
}

unsafe impl Sync for TaskManager {}
const DEFAULT_STRIDE : usize = BIG_STRIDE / 2;
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock { task_cx_ptr: 0, task_status: TaskStatus::UnInit, task_stride: DEFAULT_STRIDE };
            MAX_APP_NUM
        ];
        let mut task_heap: BinaryHeap<(Pass, usize), Min, MAX_APP_NUM> = BinaryHeap::new();
        for i in 0..num_app {
            tasks[i].task_cx_ptr = init_app_cx(i) as * const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
            tasks[i].task_stride = DEFAULT_STRIDE;
            task_heap.push((Pass(DEFAULT_STRIDE), i)).unwrap();
        }
        TaskManager {
            num_app,
            inner: RefCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
                task_heap: task_heap,
            }),
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) {
        self.inner.borrow_mut().tasks[0].task_status = TaskStatus::Running;
        let next_task_cx_ptr2 = self.inner.borrow().tasks[0].get_task_cx_ptr2();
        let _unused: usize = 0;
        unsafe {
            __switch(
                &_unused as *const _,
                next_task_cx_ptr2,
            );
        }
    }

    fn set_current_priority(&self, prio : usize) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_stride = BIG_STRIDE / prio;
        println!("Set process {} with priority {} task stride to {}", current, prio, inner.tasks[current].task_stride);
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        // Assume all tasks are either READY or EXITED, no UnInit or Running
        let mut inner = self.inner.borrow_mut();
        while let Some((pass, process_id)) = inner.task_heap.pop() {
            if inner.tasks[process_id].task_status == TaskStatus::Ready {
                let stride : usize = inner.tasks[process_id].task_stride;
                // println!("current_pid={},next_pid={}, next_pass={:?}, next_stride={}, process_heap_size={:?}\n", inner.current_task, process_id, pass, stride, inner.task_heap);
                // println!("current_pid={},next_pid={}, next_pass={:?}, next_stride={}\n", inner.current_task, process_id, pass, stride);

                let next_pass : Pass = pass.add_stride(stride);
                inner.task_heap.push((next_pass, process_id));
                return Some(process_id)
            }
        }
        None
        // let inner = self.inner.borrow();
        // let current = inner.current_task;
        // (current + 1..current + self.num_app + 1)
        //     .map(|id| id % self.num_app)
        //     .find(|id| {
        //         inner.tasks[*id].task_status == TaskStatus::Ready
        //     })
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr2 = inner.tasks[current].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();
            core::mem::drop(inner);
            unsafe {
                __switch(
                    current_task_cx_ptr2,
                    next_task_cx_ptr2,
                );
            }
        } else {
            panic!("All applications completed!");
        }
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn set_current_priority(prio : usize) {
    TASK_MANAGER.set_current_priority(prio);
}


pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

use core::cmp::Ordering;

#[derive(Eq)]
#[derive(Debug)]
struct Pass(usize);

impl Pass {
    fn add_stride(self, stride : usize) -> Pass {
        Pass(self.0.wrapping_add(stride))
    }
}

impl Ord for Pass {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            if other.0 - self.0 > BIG_STRIDE / 2 {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        } else if self.0 > other.0 {
            if self.0 - other.0 > BIG_STRIDE / 2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Pass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pass {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}