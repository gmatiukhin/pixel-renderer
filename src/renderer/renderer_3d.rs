use crate::{
    camera::Camera,
    drawing::{LineBuilder, Pixel, Shape2D, WuLine},
    renderer::{Drawifier, Renderer},
};
use glam::{Mat4, Vec3, Vec4};
use itertools::Itertools;
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

        let shapes = objects
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

                let planes = o
                    .indices()
                    .iter()
                    .flat_map(|t| {
                        let p0 = points[t.0];
                        let p1 = points[t.1];
                        let p2 = points[t.2];

                        let min = (p0.0.min(p1.0.min(p2.0)), p0.1.min(p1.1.min(p2.1)));

                        let max = (p0.0.max(p1.0.max(p2.0)), p0.1.max(p1.1.max(p2.1)));

                        (min.0..max.0)
                            .cartesian_product(min.1..max.1)
                            .flat_map(|(x, y)| {
                                let area =
                                    edge_function((p0.0, p0.1), (p1.0, p1.1), (p2.0, p2.1)) as f32;
                                let w0 = edge_function((p1.0, p1.1), (p2.0, p2.1), (x, y));
                                let w1 = edge_function((p2.0, p2.1), (p0.0, p0.1), (x, y));
                                let w2 = edge_function((p0.0, p0.1), (p1.0, p1.1), (x, y));
                                if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                                    let w0 = w0 as f32 / area;
                                    let w1 = w1 as f32 / area;
                                    let w2 = w2 as f32 / area;
                                    let c = Srgba::new(w0, w1, w2, 1f32);
                                    Some(Shape2D::Pixel(Pixel { x, y, color: c }))
                                } else {
                                    None
                                }
                            })
                            .collect_vec()
                    })
                    .collect_vec();

                let lines = o
                    .indices()
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
                    .collect_vec();

                [planes, lines]
            })
            .flatten()
            .collect_vec();

        let d = Drawifier {
            output_width: self.output_width,
            output_height: self.output_height,
        };
        d.render(camera, &shapes, frame);
    }

    fn set_output_dimensions(&mut self, width: u32, height: u32) {
        self.output_width = width;
        self.output_height = height;
    }
}

fn edge_function(a: (i32, i32), b: (i32, i32), p: (i32, i32)) -> i32 {
    // TODO: there should be no `-` sign
    -((p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0))
}
