use alloc::{collections::BTreeMap, vec::Vec};
use core::ops::Range;
use enum_repr::EnumRepr;

use crate::mprintln;

#[derive(Debug)]
#[repr(C)]
pub struct Elf64RELA {
    pub offset: usize,
    pub info: usize,
    pub addend: usize,
}

#[repr(C)]
pub struct Elf64REL {
    pub offset: usize,
    pub info: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[EnumRepr(type = "isize")]
enum DynTag {
    DT_NULL = 0,
    DT_STRTAB = 5,
    DT_SYMTAB = 6,
    DT_RELA = 7,
    DT_RELASZ = 8,
    DT_RELAENT = 9,
}

pub enum RelTable<'a> {
    RELA(&'a [Elf64RELA]),
    REL(&'a [Elf64REL]),
}

#[derive(Debug)]
#[repr(C)]
pub struct Sym {
    pub name: u32,
    pub info: u8,
    pub other: u8,
    pub shndx: u16,
    pub value: u64,
    pub size: u64,
}

pub struct Dynamic<'a> {
    pub rel: Option<RelTable<'a>>,
    pub dynsym: Option<&'a [Sym]>,
    pub dynstr: Option<&'a [u8]>,
}

#[repr(C, align(8))]
#[derive(Debug)]
struct DynEnt {
    tag: isize,
    val: usize,
}

impl<'a> Dynamic<'a> {
    pub fn parse(elf: &'a [u8], dynamic: Range<usize>) -> Self {
        mprintln!("[Linker] dynamic at {} -> {}", dynamic.start, dynamic.end);
        let (_, dynamic_region, _) = unsafe { elf[dynamic].align_to::<DynEnt>() };
        let collected: BTreeMap<DynTag, usize> = dynamic_region
            .iter()
            .take_while(|e| e.tag != 0)
            .filter_map(|e| DynTag::from_repr(e.tag).map(|tag| (tag, e.val)))
            .collect();
        mprintln!("[Linker] dynamic: {:#?}", collected);

        let mut result = Self {
            rel: None,
            dynsym: None,
            dynstr: None,
        };
        if let Some(addr) = collected.get(&DynTag::DT_RELA) {
            let sz = collected.get(&DynTag::DT_RELASZ).unwrap();
            let ent = collected.get(&DynTag::DT_RELAENT).unwrap();
            mprintln!("Rela: base {:#x}, sz {:#x}, ent {:#x}", addr, sz, ent);
            let rela = unsafe {
                core::slice::from_raw_parts(
                    &elf[*addr] as *const u8 as *const Elf64RELA,
                    *sz / *ent,
                )
            };
            result.rel = Some(RelTable::RELA(rela));
        }

        if let Some(addr) = collected.get(&DynTag::DT_SYMTAB) {
            result.dynsym = Some(unsafe {
                core::slice::from_raw_parts(
                    &elf[*addr] as *const u8 as *const Sym,
                    (elf.len() - addr) / core::mem::size_of::<Sym>(),
                )
            });
        }

        if let Some(addr) = collected.get(&DynTag::DT_STRTAB) {
            result.dynstr = Some(&elf[*addr..]);
        }

        result
    }

    pub fn resolve_sym(&self, idx: usize) -> (&Sym, &[u8]) {
        let sym = &self.dynsym.unwrap()[idx];
        let str_start = &self.dynstr.unwrap()[sym.name as usize..];
        let name = str_start.split(|e| *e == 0).next().unwrap();
        (sym, name)
    }
}
