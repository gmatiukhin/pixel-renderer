use glam::Vec3;

use crate::{
    camera::Camera,
    drawing::{LineBuilder, WuLine},
    renderer::{Drawifier, Renderer},
};

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
        let lines = objects
            .iter()
            .flat_map(|o| {
                let points = o
                    .vertices()
                    .iter()
                    .map(|v| {
                        println!("Initial points: {}, {}, {}", v.x, v.y, v.z);
                        // Project points onto the canvas
                        let x_proj = (v.x / (-v.z)) * camera.near;
                        let y_proj = (v.y / (-v.z)) * camera.near;
                        println!("Canvas projection: {x_proj}, {y_proj}");

                        // Remap points into NDC (Normalized Device Coordinates) space.
                        // Basically normalize points to [0; 1]
                        let x_proj_normal = ((canvas.width / 2f32) + x_proj) / canvas.width;
                        let y_proj_normal = ((canvas.height / 2f32) + y_proj) / canvas.height;
                        println!("NDC: {x_proj_normal}, {y_proj_normal}");

                        // Project normalized coordinates to raster space
                        let x_pix = (x_proj_normal * self.output_width as f32) as i32;
                        let y_pix = (y_proj_normal * self.output_height as f32) as i32;
                        println!("Image space: {x_pix}, {y_pix}");
                        (x_pix, y_pix)
                    })
                    .collect::<Vec<_>>();
                let mut index_ordered_vertices = vec![];
                for (v1, v2, v3) in o.indices() {
                    index_ordered_vertices.push(points[v1]);
                    index_ordered_vertices.push(points[v2]);
                    index_ordered_vertices.push(points[v3]);
                }

                if index_ordered_vertices.len() >= 3 {
                    let mut line_builder = LineBuilder::<WuLine>::new()
                        .from(index_ordered_vertices[0])
                        .to(index_ordered_vertices[1])
                        .to(index_ordered_vertices[2])
                        .close();
                    for t in index_ordered_vertices.chunks_exact(3).skip(1) {
                        line_builder = line_builder.from(t[0]).to(t[1]).to(t[2]).close();
                    }
                    Some(line_builder.end().collect::<Vec<_>>())
                } else {
                    None
                }
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
