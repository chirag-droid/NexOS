use core::time::Duration;

use alloc::vec::Vec;
use uefi::{Char16, Event};
use uefi_services::system_table;

pub fn get_key_press() -> Option<char> {
    use slint::platform::Key::*;
    use uefi::proto::console::text::Key as UefiKey;
    use uefi::proto::console::text::ScanCode as Scan;

    let nl = Char16::try_from('\r').unwrap();

    match system_table().stdin().read_key() {
        Err(_) | Ok(None) => None,
        Ok(Some(UefiKey::Printable(key))) if key == nl => Some('\n'),
        Ok(Some(UefiKey::Printable(key))) => Some(char::from(key)),
        Ok(Some(UefiKey::Special(key))) => Some(
            match key {
                Scan::UP => UpArrow,
                Scan::DOWN => DownArrow,
                Scan::RIGHT => RightArrow,
                Scan::LEFT => LeftArrow,
                Scan::HOME => Home,
                Scan::END => End,
                Scan::INSERT => Insert,
                Scan::DELETE => Delete,
                Scan::PAGE_UP => PageUp,
                Scan::PAGE_DOWN => PageDown,
                Scan::ESCAPE => Escape,
                Scan::FUNCTION_1 => F1,
                Scan::FUNCTION_2 => F2,
                Scan::FUNCTION_3 => F3,
                Scan::FUNCTION_4 => F4,
                Scan::FUNCTION_5 => F5,
                Scan::FUNCTION_6 => F6,
                Scan::FUNCTION_7 => F7,
                Scan::FUNCTION_8 => F8,
                Scan::FUNCTION_9 => F9,
                Scan::FUNCTION_10 => F10,
                Scan::FUNCTION_11 => F11,
                Scan::FUNCTION_12 => F12,
                Scan::FUNCTION_13 => F13,
                Scan::FUNCTION_14 => F14,
                Scan::FUNCTION_15 => F15,
                Scan::FUNCTION_16 => F16,
                Scan::FUNCTION_17 => F17,
                Scan::FUNCTION_18 => F18,
                Scan::FUNCTION_19 => F19,
                Scan::FUNCTION_20 => F20,
                Scan::FUNCTION_21 => F21,
                Scan::FUNCTION_22 => F22,
                Scan::FUNCTION_23 => F23,
                Scan::FUNCTION_24 => F24,
                _ => return None,
            }
            .into(),
        ),
    }
}

pub fn wait_for_input(max_timeout: Option<Duration>) {
    use uefi::table::boot::*;

    let watchdog_timeout = Duration::from_secs(120);
    let timeout = watchdog_timeout.min(max_timeout.unwrap_or(watchdog_timeout));

    let binding = system_table();
    let bs = binding.boot_services();

    // SAFETY: The event is closed before returning from this function.
    let timer = unsafe {
        bs.create_event(EventType::TIMER, Tpl::APPLICATION, None, None)
            .unwrap()
    };
    bs.set_timer(
        &timer,
        TimerTrigger::Periodic((timeout.as_nanos() / 100) as u64),
    )
    .unwrap();

    bs.set_watchdog_timer(2 * watchdog_timeout.as_micros() as usize, 0x10000, None)
        .unwrap();

    {
        // SAFETY: The cloned handles are only used to wait for further input events and
        // are then immediately dropped.
        let events = unsafe {
            [
                system_table().stdin().wait_for_key_event(),
                Some(timer.unsafe_clone()),
            ]
        };
        let mut events: Vec<Event> = events.into_iter().flatten().collect();
        bs.wait_for_event(&mut events).unwrap();
    }

    bs.set_watchdog_timer(2 * watchdog_timeout.as_micros() as usize, 0x10000, None)
        .unwrap();
    bs.close_event(timer).unwrap();
}
