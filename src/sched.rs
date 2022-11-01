use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::mprintln;
use crate::process::Process;
use crate::trap::TrapFrame;

lazy_static! {
    static ref SCHEDULER: Mutex<Sched> = Mutex::new(Sched::new());
}

pub struct Sched {
    processes: BTreeMap<usize, Process>,
    ready: VecDeque<usize>,
    running: usize,
}

impl Sched {
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            ready: VecDeque::new(),
            running: 0,
        }
    }
}

pub fn bootstrap(init: Process) {
    let mut sched = SCHEDULER.lock();
    sched.processes.insert(1, init);
    sched.running = 1;

    let proc = sched.processes.get(&1).unwrap();

    proc.mset.activate();

    // Reset stack
    const TF_SIZE: usize = core::mem::size_of::<TrapFrame>();
    let tf_push = unsafe {
        crate::INIT_STACK
            .as_mut_ptr()
            .offset((crate::consts::KERNEL_STACK_SIZE - TF_SIZE) as isize)
    };

    unsafe {
        (tf_push as *mut TrapFrame).write(proc.tf.clone());
        kickoff_init(tf_push as usize);
    }
}

unsafe fn kickoff_init(sp: usize) -> ! {
    mprintln!(
        "Jumping to process! new sp = {:#x}, sp top = {:#x}, kstack top = {:#x}",
        sp,
        sp + core::mem::size_of::<TrapFrame>(),
        crate::INIT_STACK
            .as_ptr()
            .offset(crate::consts::KERNEL_STACK_SIZE as isize) as usize
    );
    core::arch::asm!(
        "mv sp, a0",
        "j trap_exit",
        in("a0") sp,
        options(noreturn),
    )
}
