use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::mprintln;
use crate::process::Process;
use crate::trap::TrapFrame;

lazy_static! {
    pub static ref SCHEDULER: Mutex<Sched> = Mutex::new(Sched::new());
}

pub struct Sched {
    processes: BTreeMap<usize, Process>,
    ready: VecDeque<usize>,
    // TODO: thread local running
    running: usize,
    next_pid: AtomicUsize,
}

impl Sched {
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            ready: VecDeque::new(),
            running: 0,
            next_pid: AtomicUsize::new(1),
        }
    }

    pub fn tick(&mut self, involuntary: bool, tf: &mut TrapFrame) {
        if self.running == 0 {
            mprintln!("[Sched] Not bootstrapped yet");
            return;
        }

        mprintln!("[Sched] Currently running: {}", self.running);
        self.processes.get_mut(&self.running).unwrap().tf = tf.clone();
        if involuntary {
            self.ready.push_back(self.running);
        }

        // TODO: idle process
        self.running = self.ready.pop_front().unwrap();
        mprintln!("[Sched] Switching to: {}", self.running);
        let next = self.processes.get(&self.running).unwrap();
        next.mset.activate();
        *tf = next.tf.clone();
    }

    pub fn running_process(&mut self) -> &mut Process {
        self.processes.get_mut(&self.running).unwrap()
    }
}

pub fn push(proc: Process) {
    let mut sched = SCHEDULER.lock();
    let pid = sched.next_pid.fetch_add(1, Ordering::Relaxed);
    sched.processes.insert(pid, proc);
    sched.ready.push_back(pid);
}

pub fn tick(tf: &mut TrapFrame, involuntary: bool) {
    let mut sched = SCHEDULER.lock();
    sched.tick(involuntary, tf);
}

pub fn bootstrap() {
    let mut sched = SCHEDULER.lock();
    sched.running = sched.ready.pop_front().unwrap();

    let proc = sched.processes.get(&sched.running).unwrap();

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
        drop(sched);
        mprintln!("[Sched] Bootstrap");
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