use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics::pixelcolor::{IntoStorage, Rgb888};
use embedded_graphics::Pixel;

use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::table::boot::ScopedProtocol;

#[derive(Debug)]
pub struct DisplayError;

/// A display to represent the graphics output
/// We directly work on the gop to use the protocol features.
pub struct UefiDisplay<'a, 'b> {
    /// The protol to the Graphics Output
    protocol: &'b mut ScopedProtocol<'a, GraphicsOutput>,
}

impl<'a, 'b> UefiDisplay<'a, 'b> {
    pub fn new(protocol: &'b mut ScopedProtocol<'a, GraphicsOutput>) -> Self {
        UefiDisplay { protocol }
    }
}

impl<'a, 'b> OriginDimensions for UefiDisplay<'a, 'b> {
    fn size(&self) -> Size {
        let resolution = self.protocol.current_mode_info().resolution();
        Size::new(resolution.0 as u32, resolution.1 as u32)
    }
}

impl<'a, 'b> DrawTarget for UefiDisplay<'a, 'b> {
    type Color = Rgb888;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> core::prelude::v1::Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        // TODO: Support multiple colors and write directly to framebuffer
        // Is there performance issues for calling blit again and again??
        for Pixel(coord, color) in pixels.into_iter() {
            let result = self.protocol.blt(BltOp::BufferToVideo {
                buffer: &[BltPixel::from(color.into_storage())],
                src: BltRegion::Full,
                dest: (coord.x as usize, coord.y as usize),
                dims: (1, 1),
            });

            if result.is_err() {
                return Err(DisplayError);
            }
        }

        Ok(())
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> core::prelude::v1::Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());

        if area.is_zero_sized() {
            return Ok(());
        }

        let result = self.protocol.blt(BltOp::VideoFill {
            color: BltPixel::from(color.into_storage()),
            dest: (area.top_left.x as usize, area.top_left.y as usize),
            dims: (area.size.width as usize, area.size.height as usize),
        });

        if result.is_ok() {
            return Ok(());
        }

        Err(DisplayError)
    }
}