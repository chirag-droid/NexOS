#![no_main]
#![no_std]

use uefi::prelude::*;

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    uefi_services::println!("Hello  World");

    system_table.boot_services().stall(10_000_000);

    Status::SUCCESS
}

