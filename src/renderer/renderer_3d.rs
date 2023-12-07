use crate::{
    camera::Camera,
    drawing::{LineBuilder, WuLine},
    renderer::{Drawifier, Renderer},
};
use glam::{Mat4, Vec3, Vec4};
use palette::Srgba;

pub trait Mesh3D {
    /// An array of vertices
    fn vertices(&self) -> Vec<Vec3>;
    /// An array of triangles, formed by vertices with indices in tuples
    fn indices(&self) -> Vec<(usize, usize, usize)>;
}

pub struct Rasterizer {
    pub output_width: u32,
    pub output_height: u32,
}

impl Renderer for Rasterizer {
    type Renderable = Box<dyn Mesh3D>;

    fn render(&self, camera: &Camera, objects: &[Self::Renderable], frame: &mut [&mut [u8]]) {
        let canvas = camera.canvas((self.output_width, self.output_height));
        let world_to_camera = camera.transform.inverse();

        let perspective = Mat4::from_cols(
            Vec4::X * 2f32 * camera.near / canvas.width,
            Vec4::Y * 2f32 * camera.near / canvas.height,
            Vec4::NEG_Z * (camera.far + camera.near) / (camera.far - camera.near) + Vec4::NEG_W,
            Vec4::NEG_Z * 2f32 * camera.far * camera.near / (camera.far - camera.near),
        );

        let lines = objects
            .iter()
            .flat_map(|o| {
                let points = o
                    .vertices()
                    .iter()
                    .map(|v| {
                        println!("============================================");
                        println!("World space: {v}");

                        // // Project points onto the canvas
                        // let x_screen = (v.x / (-v.z)) * camera.near;
                        // let y_screen = (v.y / (-v.z)) * camera.near;
                        // println!("Screen space: {x_screen}, {y_screen}");
                        // // Remap points into NDC (Normalized Device Coordinates) space [-1; 1].
                        // let x_ndc = (2f32 * v.x) / canvas.width;
                        // let y_ndc = (2f32 * v.y) / canvas.height;
                        // println!("NDC: {x_ndc}, {y_ndc}");

                        // Important: point is now in homogenous coordinates
                        let v = world_to_camera * Vec4::from((*v, 1f32));
                        println!("Camera space: {v}");

                        // Apply projection, this also squishes z into [0; 1]
                        let v = perspective * v;
                        println!("Projected: {v}");

                        // Transform back from homogenous coordinates
                        let v = Vec3::new(v.x / v.w, v.y / v.w, v.z / v.w);
                        println!("Non-homogenous: {v}");

                        // Project normalized coordinates to raster space
                        let x_raster = ((v.x + 1f32) / 2f32 * self.output_width as f32) as i32;
                        // Y is down in raster space but up in NDC, so invert it
                        let y_raster = ((1f32 - v.y) / 2f32 * self.output_height as f32) as i32;
                        println!("Raster space: {x_raster}, {y_raster}");

                        // Keep z coordinate for z-buffering
                        (x_raster, y_raster, v.z)
                    })
                    .collect::<Vec<_>>();

                o.indices()
                    .iter()
                    .map(|t| {
                        LineBuilder::<WuLine>::new()
                            .color(Srgba::new(0.7f32, 0.5f32, 0.6f32, 1f32))
                            .from((points[t.0].0, points[t.0].1))
                            .to((points[t.1].0, points[t.1].1))
                            .to((points[t.2].0, points[t.2].1))
                            .close()
                            .shape()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let d = Drawifier {
            output_width: self.output_width,
            output_height: self.output_height,
        };
        d.render(camera, &lines, frame);
    }

    fn set_output_dimensions(&mut self, width: u32, height: u32) {
        self.output_width = width;
        self.output_height = height;
    }
}
