use pixel_renderer::{
    drawing::{LineBuilder, WuLine},
    renderer::{Camera, Mesh3D, Rasterizer, World},
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
                (1, 2, 3),
                // Left
                (4, 5, 6),
                (5, 6, 7),
                // Top
                (2, 3, 6),
                (3, 6, 7),
                // Bottom
                (0, 1, 4),
                (1, 4, 5),
                // Near
                (1, 3, 5),
                (3, 5, 7),
                // Far
                (0, 2, 4),
                (2, 4, 6),
            ]
        }
    }

    let _c = Cube;

    let _l = LineBuilder::<WuLine>::new()
        .from((100, 100))
        .to((200, 200))
        .to((100, 200))
        .close()
        .from((300, 100))
        .to((400, 200))
        .to((300, 200))
        .from((300, 300))
        .to((400, 400))
        .to((300, 400))
        .close()
        .shape();

    let mut world = World {
        camera: Camera {
            canvas_width: 2,
            canvas_height: 2,
            image_width: width,
            image_height: height,
            canvas_distance: 1f32,
        },
        renderer: Rasterizer,
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
            world.camera.image_width = size.width;
            world.camera.image_height = size.height;
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
