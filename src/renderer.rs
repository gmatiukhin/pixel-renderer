mod drawifier;
mod renderer_3d;

use crate::camera::Camera;
pub use drawifier::Drawifier;
pub use renderer_3d::*;

pub struct World<R: Renderer> {
    pub camera: Camera,
    pub renderer: R,
    pub objects: Vec<R::Renderable>,
}

impl<R: Renderer> World<R> {
    pub fn render(&self, frame: &mut [&mut [u8]]) {
        self.renderer.render(&self.camera, &self.objects, frame);
    }
}

pub trait Renderer {
    type Renderable;

    fn render(&self, camera: &Camera, objects: &[Self::Renderable], frame: &mut [&mut [u8]]);
    fn set_output_dimensions(&mut self, width: u32, height: u32);
}
