#![no_main]
#![no_std]

mod fs;
mod graphics;
mod input;
mod loader;
mod platform;
mod time;

extern crate alloc;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};
use loader::{load_kernel, parse_kernel};
use log::{error, info};
use platform::Platform;
use slint::platform::WindowEvent;
use uefi::{
    prelude::*,
    table::{boot::MemoryType, runtime::ResetType},
};

slint::include_modules!();

#[entry]
fn main(image_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st)
        .inspect(|_| info!("UEFI Services initialised."))
        .inspect_err(|error| error!("UEFI Services: {}", error.status()))
        .unwrap();

    let console = st.stdout();
    if let Some(mode) = console.modes().last() {
        console
            .set_mode(mode)
            .inspect_err(|e| error!("Setting best mode: {}", e.status()))
            .unwrap();
    }

    slint::platform::set_platform(Box::<Platform>::default()).unwrap();

    let ui = Main::new().unwrap();

    ui.set_firmware_vendor(String::from_utf16_lossy(st.firmware_vendor().to_u16_slice()).into());
    ui.set_firmware_version(
        format!(
            "{}.{:02}",
            st.firmware_revision() >> 16,
            st.firmware_revision() & 0xffff
        )
        .into(),
    );
    ui.set_uefi_version(st.uefi_revision().to_string().into());

    let mut buf = [0u8; 1];
    let guid = uefi::table::runtime::VariableVendor::GLOBAL_VARIABLE;
    let sb = st
        .runtime_services()
        .get_variable(cstr16!("SecureBoot"), &guid, &mut buf);
    ui.set_secure_boot(if sb.is_ok() { buf[0] == 1 } else { false });

    {
        // Don't use boot services here!
        let st = unsafe { st.unsafe_clone() };
        ui.on_reboot(move || {
            info!("Rebooting system");
            st.runtime_services()
                .reset(ResetType::COLD, Status::ABORTED, None);
        });
    }

    {
        // Don't use boot services here!
        let st = unsafe { st.unsafe_clone() };
        ui.on_shutdown(move || {
            info!("Shutting down the system");
            st.runtime_services()
                .reset(ResetType::SHUTDOWN, Status::ABORTED, None);
        });
    }

    {
        let ui_weak = ui.as_weak().unwrap();
        ui.on_close(move || {
            ui_weak.window().dispatch_event(WindowEvent::CloseRequested);
        });
    }

    ui.run().unwrap();

    // Parse kernel from file
    let kernel = parse_kernel(cstr16!("\\NexOS\\kernel"));

    // Load kernel in memory
    load_kernel(&kernel);

    let _ = st.exit_boot_services(MemoryType::LOADER_DATA);

    // Get kernel entry point and execute it
    let kstart: extern "efiapi" fn() -> ! =
        unsafe { core::mem::transmute(kernel.elf.header.pt2.entry_point()) };

    kstart();
}
