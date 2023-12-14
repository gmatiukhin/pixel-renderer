use std::f32::consts::PI;

use glam::{Mat2, Vec2};
use palette::Srgba;
use pixel_renderer::{
    camera::Camera,
    drawing::{LineBuilder, WuLine},
    renderer::{Drawifier, World},
};
use pixels::{PixelsBuilder, SurfaceTexture};
use rand::Rng;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let width = 512;
    let height = 512;
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Pixel Renderer")
        .with_resizable(false)
        .with_inner_size::<LogicalSize<i32>>((width, height).into())
        .build(&event_loop)
        .unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        PixelsBuilder::new(size.width, size.height, surface_texture)
            .enable_vsync(true)
            .blend_state(pixels::wgpu::BlendState::REPLACE)
            .build()
            .unwrap()
    };

    let mut tree = pythagoras_tree(
        5,
        (width as i32 / 2 - 50, height as i32),
        (width as i32 / 2 + 50, height as i32),
    );

    let mut shapes = vec![];

    let mut is_square = false;
    while !tree.is_empty() {
        let shape = if is_square {
            let p1 = tree.pop().unwrap();
            let p2 = tree.pop().unwrap();
            let p3 = tree.pop().unwrap();
            let p4 = tree.pop().unwrap();
            LineBuilder::<WuLine>::new()
                .from(p1)
                .to(p2)
                .to(p3)
                .to(p4)
                .close()
                .shape()
        } else {
            let p1 = tree.pop().unwrap();
            let p2 = tree.pop().unwrap();
            let p3 = tree.pop().unwrap();
            LineBuilder::<WuLine>::new()
                .from(p1)
                .to(p2)
                .to(p3)
                .close()
                .shape()
        };

        is_square = !is_square;
        shapes.push(shape);
    }

    let mut world = World {
        camera: Camera::default(),
        renderer: Drawifier {
            output_width: width,
            output_height: height,
        },
        objects: shapes,
    };

    if let Err(e) = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            elwt.exit();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            let size = size.to_logical(window.scale_factor());
            pixels
                .resize_surface(size.width, size.height)
                .expect("Error resizing pixel surface.");
            pixels
                .resize_buffer(size.width, size.height)
                .expect("Error resizing pixel buffer.");
            world.renderer.output_width = size.width;
            world.renderer.output_height = size.height;
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            let mut frame: Vec<&mut [u8]> = pixels.frame_mut().chunks_exact_mut(4).collect();
            world.render(&mut frame);
            pixels.render().expect("Error rendering frame.");
        }
        _ => (),
    }) {
        eprint!("Event loop error: {e:?}");
    }
}

fn square_from_base(p1: (i32, i32), p2: (i32, i32)) -> [(i32, i32); 4] {
    let v1 = Vec2::new(p1.0 as f32, p1.1 as f32);
    let v2 = Vec2::new(p2.0 as f32, p2.1 as f32);
    let d = v1 - v2;
    let m = d.length();
    let d = d.normalize();
    let (sin, cos) = 90f32.to_radians().sin_cos();
    let rot = Mat2::from_cols(Vec2::new(cos, sin), Vec2::new(-sin, cos));
    let r = rot * d;
    let v3 = v2 + r * m;
    let v4 = v1 + r * m;
    [
        p1,
        p2,
        (v3.x as i32, v3.y as i32),
        (v4.x as i32, v4.y as i32),
    ]
}

fn isosceles_right_angle_triangle_from_hipotenuse(
    p1: (i32, i32),
    p2: (i32, i32),
) -> [(i32, i32); 3] {
    let v1 = Vec2::new(p1.0 as f32, p1.1 as f32);
    let v2 = Vec2::new(p2.0 as f32, p2.1 as f32);
    let c = v1 - v2;
    let a = c.length() / 2f32.sqrt();
    let c = c.normalize();
    let (sin, cos) = (-45f32).to_radians().sin_cos();
    let rot = Mat2::from_cols(Vec2::new(cos, sin), Vec2::new(-sin, cos));
    let r = rot * c;
    let v3 = r * a + v2;

    [p2, (v3.x as i32, v3.y as i32), p1]
}

fn pythagoras_tree(iters: u32, p1: (i32, i32), p2: (i32, i32)) -> Vec<(i32, i32)> {
    let mut output = vec![];

    let square = square_from_base(p1, p2);
    let triangle = isosceles_right_angle_triangle_from_hipotenuse(square[2], square[3]);
    output.extend_from_slice(&square);
    output.extend_from_slice(&triangle);

    if iters == 0 {
        return output;
    }

    let tree_l = pythagoras_tree(iters - 1, triangle[0], triangle[1]);
    let tree_r = pythagoras_tree(iters - 1, triangle[1], triangle[2]);

    output.extend_from_slice(&tree_l);
    output.extend_from_slice(&tree_r);

    output
}
