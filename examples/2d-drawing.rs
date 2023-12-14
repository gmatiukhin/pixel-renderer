use palette::Srgba;
use pixel_renderer::{
    camera::Camera,
    drawing::{BresenhamCircle, Circle, LineBuilder, WuLine},
    renderer::{Drawifier, World},
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
        .with_resizable(true)
        .with_inner_size::<LogicalSize<i32>>((width, height).into())
        .build(&event_loop)
        .unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        PixelsBuilder::new(size.width, size.height, surface_texture)
            .enable_vsync(true)
            .blend_state(pixels::wgpu::BlendState::REPLACE)
            .build()
            .unwrap()
    };

    let mut world = World {
        camera: Camera::default(),
        renderer: Drawifier {
            output_width: width,
            output_height: height,
        },
        objects: vec![
            BresenhamCircle::new((200, 200), 100, Srgba::new(1f32, 0f32, 0f32, 1f32)).into(),
            BresenhamCircle::new((350, 220), 40, Srgba::new(0.6f32, 1f32, 0.9f32, 1f32)).into(),
            LineBuilder::<WuLine>::new()
                .from((130, 400))
                .to((160, 305))
                .to((190, 400))
                .close()
                .from((170, 340))
                .to((190, 320))
                .to((240, 400))
                .shape(),
        ],
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
