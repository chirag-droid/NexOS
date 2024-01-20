use log::info;
use uefi::{
    table::boot::{AllocateType, MemoryType, PAGE_SIZE},
    CStr16,
};
use x86_64::PhysAddr;
use xmas_elf::{program::Type, ElfFile};

use crate::fs::load_file;

pub struct Kernel<'a> {
    pub elf: ElfFile<'a>,
    pub size: usize,
    pub ptr: u64,
}

pub fn parse_kernel(filename: &CStr16) -> Kernel {
    let kernel = load_file(filename).expect("Failed to load kernel!");

    let elf = ElfFile::new(kernel).expect("Failed to parse ELF file.");
    let size = kernel.len();
    let ptr = kernel.as_ptr() as u64;

    Kernel { elf, size, ptr }
}

pub fn load_kernel(kernel: &Kernel) {
    let st = uefi_services::system_table();
    let bs = st.boot_services();

    let offset = PhysAddr::new(&kernel.elf.input[0] as *const u8 as u64);
    if !offset.is_aligned(PAGE_SIZE as u64) {
        panic!("ELF file is not correctly aligned");
    }

    // Somehow this code works
    for segment in kernel.elf.program_iter() {
        if let Ok(Type::Load) = segment.get_type() {
            info!("Loading segment: {:x?}", segment);
            let pages = ((segment.mem_size() - 1) / PAGE_SIZE as u64) + 1;
            let address = segment.virtual_addr() & !(PAGE_SIZE as u64 - 1);

            // TODO: This returns an error when allocating page at the same address
            // Need a better way to map memory
            // The error is not handled!!!
            info!("Allocating {pages}pages at {:x}", address);
            bs.allocate_pages(
                AllocateType::Address(address),
                MemoryType::LOADER_DATA,
                pages as usize,
            );

            // Write the Kernel to the address
            unsafe {
                core::ptr::copy_nonoverlapping(
                    (kernel.ptr + segment.offset()) as *const u8,
                    address as *mut u8,
                    segment.file_size() as usize,
                )
            }
        }
    }
}
