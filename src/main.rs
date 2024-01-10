#![no_main]
#![no_std]

use log::info;

use uefi::proto::console::text::{Key, ScanCode, Input};
use uefi::table::{SystemTable, Boot};
use uefi::table::boot::BootServices;
use uefi::{entry, Handle, Status, Result, ResultExt};

fn print_tables(system_table: &SystemTable<Boot>) {
    system_table.config_table().iter().enumerate().for_each(|table| {
        info!("{}: GUID: {}", table.0, table.1.guid);
    });
}

fn read_keyboard_events(boot_services: &BootServices, input: &mut Input) -> Result {
    loop {
        // Pause until a keyboard event occurs.
        let mut events = [input.wait_for_key_event().unwrap()];
        boot_services
            .wait_for_event(&mut events)
            .discard_errdata()?;

        match input.read_key()? {
            Some(Key::Printable(key)) => info!("Received key input: {}", key),
            Some(Key::Special(key)) => if key == ScanCode::ESCAPE { break; },
            None => ()
        }
    }

    Ok(())
}

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    {
        let best_mode = system_table.stdout().modes().last().expect("No text mode was attached");
        system_table.stdout().set_mode(best_mode).expect("Couldn't set best mode.");
    }

    info!("Welcome to NexOS!");
    print_tables(&system_table);

    {
        let bt = system_table.boot_services();
        let mut unsafe_st = unsafe { system_table.unsafe_clone() };
        let input = unsafe_st.stdin();
        read_keyboard_events(bt, input).expect("Encoutered an error while reading key events");
    }

    Status::ABORTED
}
