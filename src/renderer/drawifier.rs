use crate::drawing::{Pixel, Shape2D};
use palette::{blend::Compose, Srgba};

use super::Renderer;

/// If a rendrer renders why doesn't a drawer draw?
pub struct Drawifier {
    pub output_width: u32,
    pub output_height: u32,
}

impl Renderer for Drawifier {
    type Renderable = Shape2D;

    fn render(
        &self,
        _camera: &super::Camera,
        objects: &[Self::Renderable],
        frame: &mut [&mut [u8]],
    ) {
        for pixel in &mut *frame {
            let rgba = [0, 0, 0, 0xff];
            pixel.copy_from_slice(&rgba);
        }

        for p in objects.iter().flatten() {
            let (x, y, a) = match *p {
                Pixel::Normal { x, y } => (x, y, 0xff),
                Pixel::AntiAliased { x, y, a } => (x, y, a),
            };
            if x < 0 || y < 0 {
                continue;
            }
            let idx = self.output_width as usize * y as usize + x as usize;
            if idx >= frame.len() {
                // Indices go out of bounds only if Wu's line endpoints lie directly in the
                // bottom right corner. Hightly unlikely to happen often so we can just ignore
                // them.
                continue;
            }
            if x >= self.output_width as i32 || y >= self.output_height as i32 {
                continue;
            }
            let dest = &frame[idx];
            let dest: Srgba<f32> = Srgba::new(dest[0], dest[1], dest[2], dest[3]).into_format();
            let src: Srgba<f32> = Srgba::new(0xff_u8, 0xff_u8, 0xff_u8, a).into_format();
            let dest: [u8; 4] = src.over(dest).into_format().into();
            frame[idx].copy_from_slice(&dest);
        }
    }

    fn set_output_dimensions(&mut self, width: u32, height: u32) {
        self.output_width = width;
        self.output_height = height;
    }
}
