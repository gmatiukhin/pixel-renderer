use glam::{Mat4, Vec4};
use palette::Srgb;
use pixel_renderer::{
    camera::{Camera, FitStrategy},
    renderer::{Mesh3D, Rasterizer, VertexAttribute, World},
};
use pixels::{PixelsBuilder, SurfaceTexture};
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

    struct Cube;

    impl Mesh3D for Cube {
        fn vertices(&self) -> Vec<glam::Vec3> {
            vec![
                (2f32, -2f32, -5f32).into(),
                (2f32, -2f32, -3f32).into(),
                (2f32, 2f32, -5f32).into(),
                (2f32, 2f32, -3f32).into(),
                (-1f32, -2f32, -5f32).into(),
                (-1f32, -2f32, -3f32).into(),
                (-1f32, 2f32, -5f32).into(),
                (-1f32, 2f32, -3f32).into(),
            ]
        }

        fn indices(&self) -> Vec<(usize, usize, usize)> {
            // Sides
            vec![
                // Right
                (0, 1, 2),
                (1, 3, 2),
                // Left
                (4, 6, 5),
                (5, 6, 7),
                // Top
                (2, 3, 6),
                (3, 7, 6),
                // Bottom
                (0, 4, 1),
                (1, 4, 5),
                // Near
                (1, 5, 3),
                (3, 5, 7),
                // Far
                (0, 2, 4),
                (2, 6, 4),
            ]
        }

        fn attributes(&self) -> Vec<VertexAttribute> {
            vec![
                VertexAttribute {
                    color: Srgb::new(1f32, 1f32, 1f32),
                },
                VertexAttribute {
                    color: Srgb::new(1f32, 0.5f32, 1f32),
                },
                VertexAttribute {
                    color: Srgb::new(0f32, 1f32, 0.5f32),
                },
                VertexAttribute {
                    color: Srgb::new(0.5f32, 0f32, 1f32),
                },
                VertexAttribute {
                    color: Srgb::new(1f32, 0f32, 1f32),
                },
                VertexAttribute {
                    color: Srgb::new(0f32, 1f32, 1f32),
                },
                VertexAttribute {
                    color: Srgb::new(0f32, 0f32, 0f32),
                },
                VertexAttribute {
                    color: Srgb::new(1f32, 0f32, 0f32),
                },
            ]
        }
    }

    let _c = Cube;

    let mut world = World {
        camera: Camera {
            aperture: (35, 24),
            focal_length: 10f32,
            near: 0.1f32,
            far: 10f32,
            fit_strategy: FitStrategy::Overscan,
            transform: Mat4 {
                x_axis: Vec4::X,
                y_axis: Vec4::Y,
                z_axis: Vec4::Z,
                w_axis: Vec4::W,
            },
        },
        renderer: Rasterizer {
            output_width: width,
            output_height: height,
        },
        objects: vec![Box::new(_c)],
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
