#![no_main]
#![no_std]

use uefi::{entry, Handle, table::{SystemTable, Boot}, Status};

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bt = system_table.boot_services();

    uefi_services::println!("Hello World");
    bt.stall(5_000_000);

    Status::SUCCESS
}
