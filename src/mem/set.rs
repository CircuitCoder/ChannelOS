use core::ops::{RangeBounds, Range};

use alloc::{collections::BTreeMap, vec::Vec};
use elf_rs::ElfFile;

use crate::consts::PHYS_MEMORY_END;

use super::{Frame, addr::{VirtPageNum, VirtAddr, PhysPageNum}, paging::{PageTable, PTEFlags}};

bitflags::bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MapArea {
    vpns: Range<VirtPageNum>,
    perm: MapPermission,
    target: MapTarget,
}

impl MapArea {
    pub fn frames(vpns: Range<VirtPageNum>, perm: MapPermission) -> Self {
        let mut frames: BTreeMap<VirtPageNum, Frame> = BTreeMap::new();
        for vpn in vpns.clone() {
            frames.insert(vpn, Frame::alloc());
        }

        Self {
            vpns,
            perm,
            target: MapTarget::Framed { frames },
        }
    }

    pub fn linear(ppns: Range<PhysPageNum>, base: VirtPageNum, perm: MapPermission) -> Self {
        let mut vpn = base;
        let mut remote = BTreeMap::new();
        for ppn in ppns.clone() {
            remote.insert(vpn.clone(), ppn);
            vpn.0 += 1;
        }
        let len = ppns.end.0 - ppns.start.0;

        Self {
            vpns: base..VirtPageNum(base.0 + len),
            perm,
            target: MapTarget::Remote { remote },
        }
    }
}

pub enum MapTarget {
    Identical,
    Framed {
        frames: BTreeMap<VirtPageNum, Frame>,
    },
    Remote {
        remote: BTreeMap<VirtPageNum, PhysPageNum>, // TODO: Frame with RC?
    },
}

pub struct MemorySet {
    pub table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.table, data);
        }
        self.areas.push(map_area);
    }

    /// Assume that no conflicts.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission
    ) {
        self.push(MapArea::new(
            start_va,
            end_va,
            MapTarget::Framed { frames: BTreeMap::new() },
            permission,
        ), None);
    }

    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        // memory_set.map_trampoline();

        extern "C" {
            fn _text_start();
            fn _text_end();
            fn _ro_start();
            fn _ro_end();
            fn _rw_start();
            fn _rw_end();
            fn _kernel_end();
        }

        // map kernel sections
        crate::mprintln!(".text [{:#x}, {:#x})", _text_start as usize, _text_end as usize);
        crate::mprintln!(".rodata [{:#x}, {:#x})", _ro_start as usize, _ro_end as usize);
        crate::mprintln!(".data + .bss [{:#x}, {:#x})", _rw_start as usize, _rw_end as usize);
        crate::mprintln!("mapping .text section");
        memory_set.push(MapArea::new(
            (_text_start as usize).into(),
            (_text_end as usize).into(),
            MapTarget::Identical,
            MapPermission::R | MapPermission::X,
        ), None);
        crate::mprintln!("mapping .rodata section");
        memory_set.push(MapArea::new(
            (_ro_start as usize).into(),
            (_ro_end as usize).into(),
            MapTarget::Identical,
            MapPermission::R,
        ), None);
        crate::mprintln!("mapping .bss + .data section");
        memory_set.push(MapArea::new(
            (_rw_start as usize).into(),
            (_rw_end as usize).into(),
            MapTarget::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        crate::mprintln!("mapping physical memory");
        memory_set.push(MapArea::new(
            (_kernel_end as usize).into(),
            PHYS_MEMORY_END.into(),
            MapTarget::Identical,
            MapPermission::R | MapPermission::W,
        ), None);

        crate::mprintln!("Mapping serial port");
        memory_set.push(MapArea::new(
            0x10000000.into(),
            0x10001000.into(),
            MapTarget::Identical,
            MapPermission::R | MapPermission::W,
        ), None);

        memory_set
    }

    pub fn activate(&self) {
        crate::mprintln!("Activating page table at {:#x}000", self.table.ppn().0);
        unsafe {
            use riscv::register::satp;
            satp::set(satp::Mode::Sv39, 0, self.table.ppn().into());
            riscv::asm::sfence_vma_all();
            crate::mprintln!("SFENCE.VMA completed");
        }
    }
}


impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        target: MapTarget,
        perm: MapPermission
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpns: start_vpn..end_vpn,
            target,
            perm,
        }
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpns.clone() {
            self.map_one(page_table, vpn);
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpns.clone() {
            self.unmap_one(page_table, vpn);
        }
    }

    pub fn copy_data(&mut self, table: &mut PageTable, data: &[u8]) {
        let mut buf = data;
        // TODO: asserts same length
        for vpn in self.vpns.clone() {
            let dst = unsafe { table.translate(vpn).unwrap().ppn().bytes_array() };
            let moved_len = dst.len().min(buf.len());
            dst[..moved_len].copy_from_slice(&buf[..moved_len]);
            buf = &buf[moved_len..];
        }
    }

    pub fn map_one(&mut self, table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = match self.target {
            MapTarget::Identical => PhysPageNum(vpn.0),
            MapTarget::Framed { ref mut frames } => {
                let frame = Frame::alloc();
                let ppn = frame.ppn();
                frames.insert(vpn, frame);
                ppn
            }
            MapTarget::Remote { ref remote } => {
                remote.get(&vpn).unwrap().clone()
            }
        };

        let pte_flags = PTEFlags::from_bits(self.perm.bits).unwrap();
        table.map(vpn, ppn, pte_flags);
    }
    pub fn unmap_one(&mut self, table: &mut PageTable, vpn: VirtPageNum) {
        match self.target {
            MapTarget::Identical => todo!(),
            MapTarget::Framed { ref mut frames } => {
                frames.remove(&vpn);
            }
            MapTarget::Remote { ref mut remote } => {
                remote.remove(&vpn);
            }
        }
        table.unmap(vpn);
    }
}