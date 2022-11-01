use elf_rs::{ElfFile, SectionHeaderFlags, SectionType};

use crate::{mem::{set::{MemorySet, MapArea, MapPermission}, addr::{VirtAddr, VirtPageNum, PhysAddr}}, consts::{USER_STACK_TOP, KERNEL_STACK_SIZE, VDSO_RESIDE}, trap::TrapFrame, mprintln, elf::Dynamic, provided::kernel_meow};

pub static TEST_PROGRAM: &'static [u8] = include_bytes!("../../user/test.elf");

pub struct Process {
    pub mset: MemorySet,
    pub tf: TrapFrame,
}

lazy_static::lazy_static! {
    static ref EXPORTED_METHODS: [(&'static [u8], usize); 1] = [
        (b"kernel_meow", kernel_meow as usize),
    ];
}

impl Process {
    pub fn new_user(elf: &[u8]) -> Process {
        let parsed = elf_rs::Elf64::from_bytes(elf).unwrap();
        let header = parsed.elf_header();

        crate::mprintln!("{:?}", header);

        let mut mset = MemorySet::new_kernel();

        let mut dynamic = None;

        // Allocate memories
        for sec_hdr in parsed.section_header_iter() {
            if sec_hdr.section_name().starts_with(b".dynamic") {
                dynamic = Some(Dynamic::parse(elf, sec_hdr.offset() as usize .. (sec_hdr.offset()  + sec_hdr.size()) as usize));
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
            mprintln!("Perm: {:?}", perm);

            let area = MapArea::frames(virt_start.into() .. virt_end.into(), perm);
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
        let text_vdso_area = MapArea::linear(text_vdso_start_ppn..text_vdso_end_ppn, text_vdso_start_vpn, MapPermission::U | MapPermission::R | MapPermission::X);
        mset.push(text_vdso_area, None);

        if let Some(dynamic) = &dynamic {
            if let Some(inner) = &dynamic.rel {
                match &inner {
                    crate::elf::RelTable::RELA(tbl) => {
                        for ent in *tbl {
                            mprintln!("Offset: {:#x}", ent.offset);
                            mprintln!("Sym: {:#x}", ent.info >> 32);
                            mprintln!("Type: {:#x}", ent.info & ((1usize << 32) - 1));

                            let (sym, name) = dynamic.resolve_sym(ent.info >> 32);
                            mprintln!("Name: {:?}", name);
                            mprintln!("Sym: {:?}", sym);

                            for &(target_name, target_at) in EXPORTED_METHODS.iter() {
                                if target_name == name {
                                    // Found, fill in GOT
                                    let target_offset = target_at - _text_vdso_start as usize;
                                    let target_vaddr = VDSO_RESIDE + target_offset;
                                    let got_vaddr = ent.offset;
                                    let got_paddr = mset.table.translate_addr(got_vaddr.into()).unwrap();
                                    mprintln!("Wring to GOT paddr {:#x}", got_paddr.0);
                                    unsafe { (got_paddr.0 as *mut usize).write(target_vaddr) };
                                }
                            }
                        }
                    },
                    crate::elf::RelTable::REL(_) => todo!(),
                }
            }
        }

        // Fixup GOT

        // Allocate user stack

        // TODO: extendable stack
        let stack_end = VirtAddr(USER_STACK_TOP).ceil();
        let stack_start = VirtPageNum(stack_end.0 - 16usize);
        let stack_area = MapArea::frames(stack_start .. stack_end, MapPermission::U | MapPermission::W | MapPermission::R);
        mset.push(stack_area, None);

        let entry = parsed.entry_point() as usize;
        mprintln!("Entry: {:#x}", entry);
        let tf = TrapFrame::with_process(true, entry, USER_STACK_TOP);

        let process = Process {
            tf,
            mset,
        };

        process
    }
}
