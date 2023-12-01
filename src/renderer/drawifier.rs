use crate::drawing::{Pixel, Shape2D};
use palette::{blend::Compose, Srgba};

use super::Renderer;

/// If a rendrer renders why doesn't a drawer draw?
pub struct Drawifier;

impl Renderer for Drawifier {
    type Renderable = Shape2D;

    fn render(
        &self,
        camera: &super::Camera,
        objects: &[Self::Renderable],
        frame: &mut [&mut [u8]],
    ) {
        for pixel in &mut *frame {
            let rgba = [0, 0xaa, 0, 0xff];
            pixel.copy_from_slice(&rgba);
        }

        for p in objects.iter().flatten() {
            let (x, y, a) = match *p {
                Pixel::Normal { x, y } => (x, y, 0xff),
                Pixel::AntiAliased { x, y, a } => (x, y, a),
            };
            let idx = camera.canvas_width as usize * y as usize + x as usize;
            if idx >= frame.len() {
                // Indices go out of bounds only if Wu's line endpoints lie directly in the
                // bottom right corner. Hightly unlikely to happen often so we can just ignore
                // them.
                continue;
            }
            let dest = &frame[idx];
            let dest: Srgba<f32> = Srgba::new(dest[0], dest[1], dest[2], dest[3]).into_format();
            let src: Srgba<f32> = Srgba::new(0xff_u8, 0xff_u8, 0xff_u8, a).into_format();
            let dest: [u8; 4] = src.over(dest).into_format().into();
            frame[idx].copy_from_slice(&dest);
        }
    }
}
