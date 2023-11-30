use palette::{blend::Compose, Srgba};
use pixel_renderer::drawing::{BresenhamLine, LineBuilder, WuLine};
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
                let rgba = [0, 0, 0, 0xff];
                pixel.copy_from_slice(&rgba);
            }

            let (w, h): (i32, i32) = window.inner_size().into();
            let o = 20;
            for p in LineBuilder::<WuLine>::start(0, 0)
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
            {
                let (x, y, a) = match p {
                    pixel_renderer::drawing::Pixel::Normal { x, y } => (x, y, 0xff),
                    pixel_renderer::drawing::Pixel::AntiAliased { x, y, a } => (x, y, a),
                };
                let idx = w as usize * y as usize + x as usize;
                if idx >= frame.len() {
                    // Indices go out of bounds only if Wu's line endpoints lie directly in the
                    // bottom right corner. Hightly unlikely to happen often so we can just ignore
                    // them.
                    continue;
                }
                let dest = &frame[idx];
                let dest: Srgba<f32> = Srgba::new(dest[0], dest[1], dest[2], dest[3]).into_format();
                let src: Srgba<f32> = Srgba::new(0xff_u8, 0xff_u8, 0xff_u8, a).into_format();
                let dest = src.over(dest);
                let dest: [u8; 4] = dest.into_format().into();
                frame[idx].copy_from_slice(&dest);
            }

            pixels.render().expect("Error drawing pixels.");
        }
        _ => (),
    }) {
        eprint!("Event loop error: {e:?}");
    }
}
