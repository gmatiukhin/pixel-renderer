use crate::{
    camera::Camera,
    drawing::{LineBuilder, Pixel, Shape2D, WuLine},
    renderer::{Drawifier, Renderer},
};
use glam::{Mat4, Vec3, Vec4};
use itertools::Itertools;
use palette::{Srgb, Srgba};

#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub color: Srgb,
}

pub trait Mesh3D {
    /// An array of vertices
    fn vertices(&self) -> Vec<Vec3>;
    /// An array of triangles, formed by vertices with indices in tuples
    fn indices(&self) -> Vec<(usize, usize, usize)>;
    /// An array of vertex attributes.
    /// Each element corresponds to a vertex in `vertices()`
    fn attributes(&self) -> Vec<VertexAttribute>;
}

pub struct Rasterizer {
    pub output_width: u32,
    pub output_height: u32,
}

impl Renderer for Rasterizer {
    type Renderable = Box<dyn Mesh3D>;

    fn render(&self, camera: &Camera, objects: &[Self::Renderable], frame: &mut [&mut [u8]]) {
        let canvas = camera.canvas((self.output_width, self.output_height));
        let world_to_camera = camera.world_to_camera();

        let perspective = Mat4::from_cols(
            Vec4::X * 2f32 * camera.near / canvas.width,
            Vec4::Y * 2f32 * camera.near / canvas.height,
            Vec4::NEG_Z * (camera.far + camera.near) / (camera.far - camera.near) + Vec4::NEG_W,
            Vec4::NEG_Z * 2f32 * camera.far * camera.near / (camera.far - camera.near),
        );

        let mut depth_buffer =
            vec![f32::INFINITY; self.output_width as usize * self.output_height as usize];
        let shapes = objects
            .iter()
            .flat_map(|o| {
                let points = o
                    .vertices()
                    .iter()
                    .map(|v| {
                        // Note: this is old version of the uncommented code below
                        // this does not use matrices but reaches the same result
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
                        // Apply projection, this also squishes z into [0; 1]
                        let v = perspective * v;
                        // Transform back from homogenous coordinates
                        let v = Vec3::new(v.x / v.w, v.y / v.w, v.z / v.w);
                        // Project normalized coordinates to raster space
                        let x_raster = ((v.x + 1f32) / 2f32 * self.output_width as f32) as i32;
                        // Y is down in raster space but up in NDC, so invert it
                        let y_raster = ((1f32 - v.y) / 2f32 * self.output_height as f32) as i32;
                        // Keep z coordinate for z-buffering
                        Vec3::new(x_raster as f32, y_raster as f32, v.z)
                    })
                    .collect::<Vec<_>>();

                let attributes = o.attributes();

                let planes = o
                    .indices()
                    .iter()
                    .flat_map(|t| {
                        let p0 = points[t.0];
                        let p1 = points[t.1];
                        let p2 = points[t.2];

                        let a0 = attributes[t.0];
                        let a1 = attributes[t.1];
                        let a2 = attributes[t.2];

                        let min = (p0.x.min(p1.x.min(p2.x)), p0.y.min(p1.y.min(p2.y)));

                        let max = (p0.x.max(p1.x.max(p2.x)), p0.y.max(p1.y.max(p2.y)));

                        (min.0 as i32..max.0 as i32)
                            .cartesian_product(min.1 as i32..max.1 as i32)
                            .filter(|(x, y)| {
                                (*x as u32) < self.output_width && (*y as u32) < self.output_height
                            })
                            .flat_map(|(x, y)| {
                                let (x, y) = (x as f32, y as f32);
                                let area = edge_function((p0.x, p0.y), (p1.x, p1.y), (p2.x, p2.y));
                                let w0 = edge_function((p1.x, p1.y), (p2.x, p2.y), (x, y));
                                let w1 = edge_function((p2.x, p2.y), (p0.x, p0.y), (x, y));
                                let w2 = edge_function((p0.x, p0.y), (p1.x, p1.y), (x, y));
                                if w0 >= 0f32 && w1 >= 0f32 && w2 >= 0f32 {
                                    // Pixel does overlap the triangle
                                    let w0 = w0 / area;
                                    let w1 = w1 / area;
                                    let w2 = w2 / area;

                                    let z = 1f32
                                        / (1f32 / p0.z * w0 + 1f32 / p1.z * w1 + 1f32 / p2.z * w2);
                                    let idx = y as usize * self.output_width as usize + x as usize;
                                    if z < depth_buffer[idx] {
                                        depth_buffer[idx] = z;
                                        let c0 = a0.color;
                                        let c1 = a1.color;
                                        let c2 = a2.color;

                                        let r = w0 * c0.red + w1 * c1.red + w2 * c2.red;
                                        let g = w0 * c0.green + w1 * c1.green + w2 * c2.green;
                                        let b = w0 * c0.blue + w1 * c1.blue + w2 * c2.blue;

                                        // Multiply by z to achieve perspective correct
                                        // interpolation of color attributes.
                                        let c = Srgba::new(r * z, g * z, b * z, 1f32);
                                        Some(Shape2D::Pixel(Pixel {
                                            x: x as i32,
                                            y: y as i32,
                                            color: c,
                                        }))
                                    } else {
                                        None
                                    }
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
                            .from((points[t.0].x as i32, points[t.0].y as i32))
                            .to((points[t.1].x as i32, points[t.1].y as i32))
                            .to((points[t.2].x as i32, points[t.2].y as i32))
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

fn edge_function(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    // TODO: there should be no `-` sign
    -((p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0))
}
