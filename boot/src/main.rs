#![no_main]
#![no_std]

mod graphics;
mod input;
mod platform;
mod time;

extern crate alloc;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};
use log::{error, info};
use platform::Platform;
use uefi::{prelude::*, table::runtime::ResetType};

slint::include_modules!();

#[entry]
fn main(_image_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st)
        .inspect(|_| info!("UEFI Services initialised."))
        .inspect_err(|error| error!("UEFI Services: {}", error.status()))
        .unwrap();

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
        let st = unsafe { st.unsafe_clone() };
        ui.on_reboot(move || {
            info!("Rebooting system");
            st.runtime_services().reset(ResetType::COLD, Status::ABORTED, None);
        });
    }

    {
        let st = unsafe { st.unsafe_clone() };
        ui.on_shutdown(move || {
            info!("Shutting down the system");
            st.runtime_services().reset(ResetType::SHUTDOWN, Status::ABORTED, None);
        });
    }

    ui.run().unwrap();

    Status::SUCCESS
}
