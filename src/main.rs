#![no_main]
#![no_std]

mod graphics;

use embedded_graphics::transform::Transform;
use graphics::Buffer;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::primitives::{Triangle, StyledDrawable, PrimitiveStyleBuilder};
use embedded_graphics::mono_font::jis_x0201::FONT_10X20;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point, Dimensions, AnchorPoint, OriginDimensions};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{RgbColor, Rgb888};
use embedded_graphics::text::{Text, Alignment};

use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::text::{Input, Key, ScanCode};
use uefi::table::boot::BootServices;
use uefi::table::{SystemTable, Boot};
use uefi::{entry, Handle, Status, Result, ResultExt};

use log::info;

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

    let bs = system_table.boot_services();

    // Open graphics output protocol.
    let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = bs.open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

    // Create a buffer
    let resolution = gop.current_mode_info().resolution();
    let mut display = Buffer::new(resolution.0, resolution.1);
    info!("Created a graphics buffer: {}x{}", display.size().width, display.size().height);

    // Fill with gray color
    display.clear(Rgb888::new(0x22, 0x22, 0x22)).unwrap();

    // Draw centered text.
    let character_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
    let text = "Welcome To NexOS!";
    Text::with_alignment(
        text,
        display.bounding_box().anchor_point(AnchorPoint::TopCenter) + Point::new(0, 20),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    // Draw a filled triangle.
    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::WHITE)
        .build();
    Triangle::new(
        display.bounding_box().anchor_point(AnchorPoint::TopCenter) / 2,
        display.bounding_box().anchor_point(AnchorPoint::BottomLeft) / 2,
        display.bounding_box().anchor_point(AnchorPoint::BottomRight) / 2,
    )
    .translate(display.bounding_box().center() / 2)
    .draw_styled(&style, &mut display)
    .unwrap();

    display.blit(&mut gop).unwrap();

    // Read Key inputs
    {
        let bt = system_table.boot_services();
        let mut unsafe_st = unsafe { system_table.unsafe_clone() };
        let input = unsafe_st.stdin();
        read_keyboard_events(bt, input).expect("Encoutered an error while reading key events");
   }

    Status::ABORTED
}
