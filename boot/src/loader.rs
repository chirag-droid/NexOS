extern crate alloc;

use alloc::{vec, vec::Vec};
use log::info;
use uefi::{
    table::boot::{AllocateType, MemoryType, PAGE_SIZE},
    CStr16,
};
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

    // This is a poor way of handling used up addresses
    // This should be handled by the code calling this function
    let mut used_pages: Vec<u64> = vec![];

    // Somehow this code works
    for segment in kernel.elf.program_iter() {
        if let Ok(Type::Load) = segment.get_type() {
            info!("Loading segment: {:x?}", segment);
            let pages = ((segment.mem_size() - 1) / PAGE_SIZE as u64) + 1;
            let address = segment.virtual_addr() & !(PAGE_SIZE as u64 - 1);

            if !used_pages.contains(&address) {
                bs.allocate_pages(
                    AllocateType::Address(address),
                    MemoryType::LOADER_DATA,
                    pages as usize,
                )
                .unwrap();

                used_pages.extend(vec![1u64; pages as usize].iter().map(|x| x * address));
            }

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
