use alloc::slice;
use log::{error, info};
use uefi::{
    proto::media::{
        file::{File, FileAttribute, FileInfo, FileMode, FileType},
        fs::SimpleFileSystem,
    },
    table::boot::{AllocateType, MemoryType, PAGE_SIZE},
    CStr16,
};
use uefi_services::system_table;

pub fn load_file(filename: &CStr16) -> Option<&'static mut [u8]> {
    let st = system_table();
    let bs = st.boot_services();

    // File system protocol. Used to laod the kernel lib
    let fs_handle = bs
        .get_handle_for_protocol::<SimpleFileSystem>()
        .expect("Handle for File System Protocol not found.");

    let mut file_system = bs
        .open_protocol_exclusive::<SimpleFileSystem>(fs_handle)
        .inspect_err(|e| error!("File System: {}", e.status()))
        .unwrap();

    // Open the root directory
    let mut root = file_system
        .open_volume()
        .inspect(|_| info!("Opened root volume!"))
        .inspect_err(|error| error!("Open root volume: {}", error.status()))
        .unwrap();

    let file_handle = root
        .open(filename, FileMode::Read, FileAttribute::empty())
        .inspect(|_| info!("{filename} loaded from fs!"))
        .inspect_err(|error| error!("Reading file: {}", error.status()))
        .unwrap();

    let mut file = match file_handle.into_type().unwrap() {
        FileType::Regular(f) => f,
        FileType::Dir(_) => panic!(),
    };

    // Get file size
    let file_size = usize::try_from(
        file.get_boxed_info::<FileInfo>()
            .expect("Failed to get file info.")
            .file_size(),
    )
    .unwrap();

    // Allocate memory for the buffer
    let file_ptr = bs
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            ((file_size - 1) / PAGE_SIZE) + 1,
        )
        .inspect(|_| info!("Allocated {file_size}B for {filename}"))
        .unwrap();

    // Read the file into the above buffer.
    let file_slice = unsafe { slice::from_raw_parts_mut(file_ptr as *mut u8, file_size) };
    file.read(file_slice).unwrap();

    Some(file_slice)
}
