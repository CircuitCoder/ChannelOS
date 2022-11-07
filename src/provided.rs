use core::{sync::atomic::{AtomicU8, Ordering, AtomicU16, AtomicBool, AtomicU32}};

use crate::{consts, service::PutcharQueue};

#[link_section = ".text.vdso"]
pub extern "C" fn kernel_meow() -> usize {
    return 0;
}

#[link_section = ".text.vdso"]
pub extern "C" fn putchar_sync(c: char) {
    unsafe {
        core::arch::asm!(
            "li a0, 0x100",
            "ecall",

            in("a1") c as usize
        )
    }
}

#[link_section = ".text.vdso"]
pub extern "C" fn putchar_async(c: char) {
    // TODO: service table
    let queue_page_ptr = consts::VDSO_DATA as *mut usize;
    let mut req_page = unsafe { queue_page_ptr.read() };
    let mut resp_page: usize = unsafe { queue_page_ptr.offset(1).read() };

    if req_page == 0 {
        let mut allocated_req: usize;
        let mut allocated_resp: usize;
        unsafe {
            core::arch::asm!(
                "li a0, 0x3", // Request service
                "li a1, 0x0", // Putchar service
                "ecall",
                out("a0") allocated_req,
                out("a1") allocated_resp,
            );
        }

        unsafe {
            queue_page_ptr.write(allocated_req);
            queue_page_ptr.offset(1).write(allocated_resp);
        }

        req_page = allocated_req;
        resp_page = allocated_resp;
    }

    let req_page = unsafe { &mut *(req_page as *mut PutcharQueue) };
    let cur_trans = req_page.trans.load(Ordering::Relaxed);
    while cur_trans - req_page.recv.load(Ordering::Acquire) >= 255 {
        // Spin
    }

    req_page.data[cur_trans as usize % 255] = (cur_trans as u64, c as u64);
    req_page.trans.fetch_add(1, Ordering::Release);
}