use alloc::collections::VecDeque;
use buddy_system_allocator::LockedHeap;

use crate::consts::*;
use riscv::register::sstatus;

use self::{addr::PhysPageNum, set::MemorySet};

pub mod addr;
pub mod paging;
pub mod set;

#[global_allocator]
static DYNAMIC_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

pub fn init() {
    unsafe {
        sstatus::set_sum();
    }
    init_heap();
    init_frame();
}

fn init_heap() {
    static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
    unsafe {
        DYNAMIC_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    panic!("alloc_error_handler do nothing but panic!");
}

// Frame allocator
extern "C" {
    fn _frames_start();
}

// TODO: OOM
enum NaiveFrameAllocator {
    Init {
        freelist: VecDeque<usize>,
        ptr: usize,
    },
    Uninit,
}

impl NaiveFrameAllocator {
    fn init(&mut self) {
        *self = Self::Init {
            freelist: VecDeque::new(),
            ptr: addr::PhysAddr::from(_frames_start as usize).ceil().into(),
        }
    }
}

static FRAME_ALLOC: spin::Mutex<NaiveFrameAllocator> =
    spin::Mutex::new(NaiveFrameAllocator::Uninit);

fn init_frame() {
    let mut lock = FRAME_ALLOC.lock();
    lock.init();
}

pub struct Frame(usize);

// TODO: OOM
impl Frame {
    pub fn alloc() -> Self {
        match *FRAME_ALLOC.lock() {
            NaiveFrameAllocator::Uninit => panic!("Frame allocated before init"),
            NaiveFrameAllocator::Init {
                ref mut freelist,
                ref mut ptr,
            } => {
                if let Some(p) = freelist.pop_front() {
                    return Self(p);
                }

                let alloc = *ptr;
                *ptr += 1;
                return Self(alloc);
            }
        }
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum::from(self.0)
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        match *FRAME_ALLOC.lock() {
            NaiveFrameAllocator::Uninit => panic!("Frame de-allocated before init"),
            NaiveFrameAllocator::Init {
                ref mut freelist, ..
            } => {
                freelist.push_front(self.0);
            }
        }
    }
}
