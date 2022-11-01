use alloc::vec;
use alloc::vec::Vec;

use crate::consts::PAGE_SIZE;

use super::{addr::*, Frame};

bitflags::bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PTE {
    pub bits: usize,
}

impl PTE {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PTE {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PTE { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

// os/src/mm/page_table.rs

pub struct PageTable {
    ppn: PhysPageNum,
    frames: Vec<Frame>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = Frame::alloc();
        let ppn = frame.ppn();
        let frames = vec![frame];

        PageTable { ppn, frames }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PTE::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PTE::empty();
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PTE> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub fn translate_addr(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        self.translate(vaddr.floor().into()).map(|pte| {
            let ppn = pte.ppn();
            let mut paddr_base: PhysAddr = ppn.into();
            paddr_base.0 += vaddr.page_offset();
            paddr_base
        })
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PTE> {
        let idxs = vpn.indexes();
        let mut ppn = self.ppn;
        let mut result: Option<&mut PTE> = None;
        for i in 0..3 {
            let pte = unsafe { ppn.pte_within(idxs[i]) };
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = Frame::alloc();
                *pte = PTE::new(frame.ppn(), PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PTE> {
        let idxs = vpn.indexes();
        let mut ppn = self.ppn;
        let mut result: Option<&mut PTE> = None;
        for i in 0..3 {
            let pte = unsafe { ppn.pte_within(idxs[i]) };
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    pub fn ppn(&self) -> PhysPageNum {
        self.ppn
    }
}
