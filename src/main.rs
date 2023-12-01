use pixel_renderer::{
    drawing::{BresenhamLine, LineBuilder, WuLine},
    renderer::{Camera, Drawifier, World},
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

    let mut world = World {
        camera: Camera {
            canvas_width: width,
            _canvas_height: height,
        },
        renderer: Drawifier,
        objects: vec![],
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
            world.camera.canvas_width = size.width;
            world.camera._canvas_height = size.height;
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            let mut frame: Vec<&mut [u8]> = pixels.frame_mut().chunks_exact_mut(4).collect();
            let (w, h): (i32, i32) = window.inner_size().into();
            let o = 20;
            let l = LineBuilder::<WuLine>::start(0, 0)
                .to(w - 1, h - 1)
                .to(w - 1, h / 2)
                .to(w / 2, 0)
                .close()
                .chain(
                    LineBuilder::<BresenhamLine>::start(0, o)
                        .to(w - 1 - o, h - 1)
                        .to(w - 1 - o, h / 2)
                        .to(w / 2, o)
                        .close(),
                )
                .collect::<Vec<_>>();
            world.objects = vec![l];
            world.render(&mut frame);
            let _ = pixels.render();
        }
        _ => (),
    }) {
        eprint!("Event loop error: {e:?}");
    }
}
