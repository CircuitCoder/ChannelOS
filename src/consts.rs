pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;
pub const KERNEL_STACK_SIZE: usize = 0x8_0000;

pub const PHYS_MEMORY_END: usize = 0x8800_0000;

pub const PAGE_SIZE: usize = 4096;

// For processes
pub const USER_STACK_TOP: usize = 0x80000000;

pub const VDSO_RESIDE: usize = 0x60000000;
