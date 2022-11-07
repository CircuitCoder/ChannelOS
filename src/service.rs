use core::{sync::atomic::{AtomicU8, Ordering, AtomicU16, AtomicBool, AtomicU32}};

use crate::{uprint, mem::{Frame, addr::{PhysAddr, VirtAddr}, set::{MapArea, MapPermission}}, process::{Process, UserCaps}, mprintln, prog};

#[repr(C)]
pub struct PutcharQueue {
    pub recv: AtomicU32,
    pub trans: AtomicU32,
    pub remote_sleeping: AtomicBool,
    pub closed: AtomicBool,

    pub data: [(u64, u64); 255],
}

fn putchar_kservice(queue: &mut PutcharQueue) -> ! {
    if core::mem::size_of::<PutcharQueue>() != 0x1000 {
        panic!("Incorrect putchar queue size: {}", core::mem::size_of::<PutcharQueue>());
    }

    loop {
        let cur = queue.recv.load(Ordering::Relaxed);
        mprintln!("Tx {}, Rx {}", queue.trans.load(Ordering::Relaxed), queue.recv.load(Ordering::Relaxed));
        if queue.trans.load(Ordering::Acquire) > cur {
            let (_, c) = queue.data[cur as usize % 255];
            queue.recv.fetch_add(1, Ordering::Relaxed);
            uprint!("{}", c as u8 as char);
        }
    }
}

fn putchar_kboot() -> (PhysAddr, PhysAddr) {
    let req = Frame::alloc();
    let resp = Frame::alloc();
    
    let req_paddr: PhysAddr = req.ppn().into();
    let resp_paddr: PhysAddr = resp.ppn().into();

    // Zeroing pages
    unsafe { *(req_paddr.0 as *mut [u8; 4096]) = [0u8; 4096] };
    unsafe { *(resp_paddr.0 as *mut [u8; 4096]) = [0u8; 4096] };

    let kservice = Process::new_kernel(putchar_kservice as usize, [req_paddr.0, resp_paddr.0]);
    crate::sched::push(kservice);

    // TODO: track frame in processes
    core::mem::forget(req);
    core::mem::forget(resp);

    (req_paddr, resp_paddr)
}

fn putchar_uboot() -> (PhysAddr, PhysAddr) {
    let req = Frame::alloc();
    let resp = Frame::alloc();
    
    let req_paddr: PhysAddr = req.ppn().into();
    let resp_paddr: PhysAddr = resp.ppn().into();

    // Zeroing pages
    unsafe { *(req_paddr.0 as *mut [u8; 4096]) = [0u8; 4096] };
    unsafe { *(resp_paddr.0 as *mut [u8; 4096]) = [0u8; 4096] };

    let mut uservice = Process::new_user(prog::PUTCHAR, [0x64000000, 0x10000000], UserCaps { serial: true });

    let req_area = MapArea::linear(
        req_paddr.floor()..PhysAddr(req_paddr.0 + 1).ceil(),
        VirtAddr(0x64000000).into(),
        MapPermission::U | MapPermission::W | MapPermission::R,
    );
    let resp_area = MapArea::linear(
        resp_paddr.floor()..PhysAddr(resp_paddr.0 + 1).ceil(),
        VirtAddr(0x64001000).into(),
        MapPermission::U | MapPermission::W | MapPermission::R,
    );
    uservice.mset.push(req_area, None);
    uservice.mset.push(resp_area, None);

    crate::sched::push(uservice);

    // TODO: track frame in processes
    core::mem::forget(req);
    core::mem::forget(resp);

    (req_paddr, resp_paddr)
}

pub const SERVICE_LIST: [fn() -> (PhysAddr, PhysAddr); 1] = [
    putchar_uboot,
];