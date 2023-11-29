use pixel_renderer::drawing::BresenhamLine;
use pixels::{Pixels, SurfaceTexture};
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
        Pixels::new(size.width, size.height, surface_texture).unwrap()
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
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            let mut frame: Vec<&mut [u8]> = pixels.frame_mut().chunks_exact_mut(4).collect();
            for pixel in &mut frame {
                let rgba = [0, 0xaa, 0, 0xff];
                pixel.copy_from_slice(&rgba);
            }

            let (width, height): (i32, i32) = window.inner_size().into();
            for (x, y) in BresenhamLine::new((0, 0), (width - 1, height - 1))
                .chain(BresenhamLine::new(
                    (width - 1, height - 1),
                    (width - 1, height / 2),
                ))
                .chain(BresenhamLine::new((width - 1, height / 2), (width / 2, 0)))
            {
                let idx = width as usize * y as usize + x as usize;
                if idx >= frame.len() {
                    println!("{x}, {y}");
                    println!("{}", idx);
                    continue;
                }
                frame[width as usize * y as usize + x as usize]
                    .copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
            }

            pixels.render().expect("Error drawing pixels.");
        }
        _ => (),
    }) {
        eprint!("Event loop error: {e:?}");
    }
}
