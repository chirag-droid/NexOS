extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::{RgbColor, Rgb888};

use uefi::proto::console::gop::{BltPixel, GraphicsOutput, BltRegion, BltOp};
use uefi::Error;
use uefi::Result;

/// This buffer works with embedded_graphics
/// and can be redirected to graphics output
pub struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<Rgb888>,
}

impl OriginDimensions for Buffer {
    fn size(&self) -> embedded_graphics::prelude::Size {
        Size {
            width: self.width as u32,
            height: self.height as u32
        }
    }
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Buffer {
            width,
            height,
            pixels: vec![RgbColor::BLACK; width * height],
        }
    }

    /// Get a single pixel.
    fn set_pixel(&mut self, i: usize, value: Rgb888) {
        self.pixels[i] = value;
    }

    /// Blit the buffer to the framebuffer.
    pub fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        let buffer: Vec<BltPixel> = self.pixels.clone().into_iter().map(|pixel| {
            BltPixel::new(pixel.r(), pixel.g(), pixel.b())
        }).collect();

        gop.blt(BltOp::BufferToVideo {
            buffer: &buffer,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width as usize, self.height as usize),
        })
    }
}

impl DrawTarget for Buffer {
    type Color = Rgb888;

    type Error = Error;

    fn draw_iter<I>(&mut self, pixels: I) -> core::prelude::v1::Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>> {
        
        pixels.into_iter().for_each(|pixel| {
            let x: usize = pixel.0.x.try_into().unwrap();
            let y: usize = pixel.0.y.try_into().unwrap();

            let i = y * self.width + x;
            self.set_pixel(i, pixel.1)
        });
        Ok(())
    }
}
