use elf_rs::{ElfFile, SectionHeaderFlags, SectionType};

use crate::{
    consts::{PROCESS_STACK_TOP, VDSO_DATA, VDSO_RESIDE},
    elf::Dynamic,
    mem::{
        addr::{PhysAddr, VirtAddr, VirtPageNum},
        set::{MapArea, MapPermission, MemorySet},
    },
    mprintln,
    provided::kernel_meow,
    provided::putchar_async,
    provided::putchar_sync,
    trap::TrapFrame,
};

pub struct Process {
    pub mset: MemorySet,
    pub tf: TrapFrame,
}

lazy_static::lazy_static! {
    static ref EXPORTED_METHODS: [(&'static [u8], usize); 2] = [
        (b"kernel_meow", kernel_meow as usize),
        // (b"putchar", putchar_sync as usize),
        (b"putchar", putchar_async as usize),
    ];
}

#[derive(Default, Clone, Copy)]
pub struct UserCaps {
    pub serial: bool,
}

impl Process {
    pub fn new_user(elf: &[u8], data: [usize; 2], caps: UserCaps) -> Process {
        let parsed = elf_rs::Elf64::from_bytes(elf).unwrap();
        let header = parsed.elf_header();

        // crate::mprintln!("{:?}", header);

        let mut mset = MemorySet::new_kernel(caps);

        let mut dynamic = None;

        // Allocate memories
        for sec_hdr in parsed.section_header_iter() {
            if sec_hdr.section_name().starts_with(b".dynamic") {
                dynamic = Some(Dynamic::parse(
                    elf,
                    sec_hdr.offset() as usize..(sec_hdr.offset() + sec_hdr.size()) as usize,
                ));
            }

            if !sec_hdr.flags().contains(SectionHeaderFlags::SHF_ALLOC) {
                continue;
            }

            crate::mprintln!("Mapping: {:?}", sec_hdr);

            let addr = sec_hdr.addr() as usize;
            let size = sec_hdr.size() as usize;
            assert!(size > 0);

            let src = if sec_hdr.sh_type() != SectionType::SHT_NOBITS {
                let offset = sec_hdr.offset() as usize;
                let content = &elf[offset..(offset + size)];
                Some(content)
            } else {
                None
            };

            let virt_start = VirtAddr(addr).floor();
            let virt_end = VirtAddr(addr + size).ceil();
            let mut perm = MapPermission::U | MapPermission::R;
            if sec_hdr.flags().contains(SectionHeaderFlags::SHF_WRITE) {
                perm |= MapPermission::W;
            }
            if sec_hdr.flags().contains(SectionHeaderFlags::SHF_EXECINSTR) {
                perm |= MapPermission::X;
            }
            // mprintln!("Perm: {:?}", perm);

            let area = MapArea::frames(virt_start.into()..virt_end.into(), perm);
            mset.push(area, src);
        }

        // Map VDSO text
        extern "C" {
            fn _text_vdso_start();
            fn _text_vdso_end();
        }
        let text_vdso_start_ppn = PhysAddr(_text_vdso_start as usize).floor();
        let text_vdso_end_ppn = PhysAddr(_text_vdso_end as usize).ceil();
        let text_vdso_start_vpn = VirtAddr(VDSO_RESIDE).floor();
        let text_vdso_area = MapArea::linear(
            text_vdso_start_ppn..text_vdso_end_ppn,
            text_vdso_start_vpn,
            MapPermission::U | MapPermission::R | MapPermission::X,
        );
        mset.push(text_vdso_area, None);

        // Map vdso data
        let data_vdso_start_vpn = VirtAddr(VDSO_DATA).floor();
        let data_vdso_frames = MapArea::frames(
            data_vdso_start_vpn..VirtPageNum(data_vdso_start_vpn.0 + 1),
            MapPermission::U | MapPermission::R | MapPermission::W,
        );
        mset.push(data_vdso_frames, Some(&[0u8; 0x1000]));

        if let Some(dynamic) = &dynamic {
            if let Some(inner) = &dynamic.rel {
                match &inner {
                    crate::elf::RelTable::RELA(tbl) => {
                        for ent in *tbl {
                            // mprintln!("Offset: {:#x}", ent.offset);
                            // mprintln!("Sym: {:#x}", ent.info >> 32);
                            let reloc_type = ent.info & ((1usize << 32) - 1);
                            // mprintln!("Type: {:#x}", reloc_type);

                            match reloc_type {
                                0x3 => {
                                    // Local variable relocation
                                    let slot_paddr =
                                        mset.table.translate_addr(ent.offset.into()).unwrap();
                                    mprintln!(
                                        "[Linker] Wring RELATIVE: {:#x}, paddr {:#x} <- {:#x}",
                                        ent.offset,
                                        slot_paddr.0,
                                        ent.addend
                                    );
                                    unsafe { (slot_paddr.0 as *mut usize).write(ent.addend) };
                                }
                                0x5 => {
                                    let (sym, name) = dynamic.resolve_sym(ent.info >> 32);
                                    // mprintln!("Name: {:?}", name);
                                    // mprintln!("Sym: {:?}", sym);

                                    for &(target_name, target_at) in EXPORTED_METHODS.iter() {
                                        if target_name == name {
                                            // Found, fill in GOT
                                            let target_offset =
                                                target_at - _text_vdso_start as usize;
                                            let target_vaddr = VDSO_RESIDE + target_offset;
                                            let got_vaddr = ent.offset;
                                            let got_paddr = mset
                                                .table
                                                .translate_addr(got_vaddr.into())
                                                .unwrap();
                                            mprintln!("[Linker] Wring JUMP_SLOT: GOT vaddr {:#x}, paddr {:#x}", got_vaddr, got_paddr.0);
                                            unsafe {
                                                (got_paddr.0 as *mut usize).write(target_vaddr)
                                            };
                                        }
                                    }
                                }
                                _ => {
                                    panic!("Unsupported relocation type {:#x}", reloc_type);
                                }
                            }
                        }
                    }
                    crate::elf::RelTable::REL(_) => todo!(),
                }
            }
        }

        // Fixup GOT

        // Allocate user stack

        // TODO: extendable stack
        let stack_end = VirtAddr(PROCESS_STACK_TOP).ceil();
        let stack_start = VirtPageNum(stack_end.0 - 16usize);
        let stack_area = MapArea::frames(
            stack_start..stack_end,
            MapPermission::U | MapPermission::W | MapPermission::R,
        );
        mset.push(stack_area, None);

        let entry = parsed.entry_point() as usize;
        mprintln!("Entry: {:#x}", entry);
        let mut tf = TrapFrame::with_process(true, entry, PROCESS_STACK_TOP);
        tf.x[10] = data[0];
        tf.x[11] = data[1];

        let process = Process { tf, mset };

        process
    }

    pub fn new_kernel(entry: usize, data: [usize; 2]) -> Process {
        let mut mset = MemorySet::new_kernel(Default::default());
        // Allocate stack

        // TODO: extendable stack
        let stack_end = VirtAddr(PROCESS_STACK_TOP).ceil();
        let stack_start = VirtPageNum(stack_end.0 - 16usize);
        let stack_area =
            MapArea::frames(stack_start..stack_end, MapPermission::W | MapPermission::R);
        mset.push(stack_area, None);

        let entry = entry as usize;
        mprintln!("Entry: {:#x}", entry);
        let mut tf = TrapFrame::with_process(false, entry, PROCESS_STACK_TOP);
        tf.x[10] = data[0];
        tf.x[11] = data[1];

        let process = Process { tf, mset };

        process
    }
}
