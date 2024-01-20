extern crate alloc;

use core::time::Duration;

use alloc::{rc::Rc, slice};
use log::error;
use slint::{platform::software_renderer, SharedString};
use uefi_services::system_table;

use crate::{
    graphics::SlintBltPixel,
    input::{get_key_press, wait_for_input},
    time::{timer_freq, timer_tick},
};

pub struct Platform {
    window: Rc<software_renderer::MinimalSoftwareWindow>,
    timer_freq: f64,
    timer_start: f64,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            window: software_renderer::MinimalSoftwareWindow::new(
                software_renderer::RepaintBufferType::ReusedBuffer,
            ),
            timer_freq: timer_freq() as f64,
            timer_start: timer_tick() as f64,
        }
    }
}

impl slint::platform::Platform for Platform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        Duration::from_secs_f64((timer_tick() as f64 - self.timer_start) / self.timer_freq)
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        use uefi::{proto::console::gop::*, table::boot::*};

        let st = system_table();
        let bs = st.boot_services();

        let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();

        // SAFETY: uefi-rs wants us to use open_protocol_exclusive(), which will not work
        // on real hardware. We can only hope that any other users of this
        // handle/protocol behave and don't interfere with our uses of it.
        let mut gop = unsafe {
            bs.open_protocol::<GraphicsOutput>(
                OpenProtocolParams {
                    handle: gop_handle,
                    agent: bs.image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .unwrap()
        };

        if let Some(mode) = gop.modes(bs).last() {
            gop.set_mode(&mode)
                .inspect_err(|e| error!("Setting best mode: {}", e.status()))
                .unwrap();
        }

        let info = gop.current_mode_info();
        let mut fb = alloc::vec![SlintBltPixel(BltPixel::new(0, 0, 0)); info.resolution().0 * info.resolution().1];

        self.window.set_size(slint::PhysicalSize::new(
            info.resolution().0.try_into().unwrap(),
            info.resolution().1.try_into().unwrap(),
        ));

        loop {
            slint::platform::update_timers_and_animations();

            while let Some(key) = get_key_press() {
                // EFI does not distinguish between pressed and released events.
                let text = SharedString::from(key);
                self.window
                    .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                        text: text.clone(),
                    });
                self.window
                    .dispatch_event(slint::platform::WindowEvent::KeyReleased { text });
            }

            self.window.draw_if_needed(|renderer| {
                renderer.render(&mut fb, info.resolution().0);

                // SAFETY: SlintBltPixel is a repr(transparent) BltPixel so it is safe to transform.
                let blt_fb =
                    unsafe { slice::from_raw_parts(fb.as_ptr() as *const BltPixel, fb.len()) };

                // We could let the software renderer draw to gop.frame_buffer() directly, but that
                // requires dealing with different frame buffer formats. The blit buffer is easier to
                // deal with and guaranteed to be available by the UEFI spec. This also reduces tearing
                // by quite a bit.
                gop.blt(BltOp::BufferToVideo {
                    buffer: blt_fb,
                    src: BltRegion::Full,
                    dest: (0, 0),
                    dims: info.resolution(),
                })
                .unwrap();
            });

            if !self.window.is_visible() {
                gop.blt(BltOp::VideoFill {
                    color: BltPixel::new(0, 0, 0),
                    dest: (0, 0),
                    dims: info.resolution(),
                })
                .unwrap();
                return Ok(());
            }

            if !self.window.has_active_animations() {
                wait_for_input(slint::platform::duration_until_next_timer_update());
            }
        }
    }
}
