use slint::platform::software_renderer;
use uefi::proto::console::gop::BltPixel;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SlintBltPixel(pub BltPixel);

impl software_renderer::TargetPixel for SlintBltPixel {
    fn blend(&mut self, color: software_renderer::PremultipliedRgbaColor) {
        let a = (u8::MAX - color.alpha) as u16;
        self.0.red = (self.0.red as u16 * a / 255) as u8 + color.red;
        self.0.green = (self.0.green as u16 * a / 255) as u8 + color.green;
        self.0.blue = (self.0.blue as u16 * a / 255) as u8 + color.blue;
    }

    fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        SlintBltPixel(BltPixel::new(red, green, blue))
    }
}
