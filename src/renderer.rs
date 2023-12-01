mod drawifier;
mod renderer_3d;

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

pub struct Camera {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub canvas_distance: f32,
}

pub trait Renderer {
    type Renderable;

    fn render(&self, camera: &Camera, objects: &[Self::Renderable], frame: &mut [&mut [u8]]);
}
