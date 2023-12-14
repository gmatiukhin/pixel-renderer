use glam::Vec3;
use palette::Srgb;
use pixel_renderer::{
    camera::{Camera, FitStrategy},
    renderer::{Mesh3D, Rasterizer, VertexAttribute, World},
};
use pixels::{PixelsBuilder, SurfaceTexture};
use radians::Rad32;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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
            position: Vec3::ZERO,
            yaw: Rad32::new(-90f32),
            pitch: Rad32::new(0f32),
        },
        renderer: Rasterizer {
            output_width: width,
            output_height: height,
        },
        objects: vec![Box::new(_c)],
    };

    let mut last_time = std::time::Instant::now();
    let mut last_cursor: Option<PhysicalPosition<f64>> = None;
    let mut lmb_pressed = false;
    if let Err(e) = event_loop.run(move |event, elwt| {
        let now = std::time::Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        match event {
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
            Event::WindowEvent { event, .. } => {
                let forward = world.camera.forward();
                let right = world.camera.right();
                let camera_speed = 10000f32 * dt;
                let sensitivity = 1000f32 * dt;

                match event {
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(key_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        match key_code {
                            // Movement
                            KeyCode::KeyW => world.camera.position += camera_speed * forward,
                            KeyCode::KeyA => world.camera.position -= camera_speed * right,
                            KeyCode::KeyS => world.camera.position -= camera_speed * forward,
                            KeyCode::KeyD => world.camera.position += camera_speed * right,
                            KeyCode::KeyQ => world.camera.position -= camera_speed * Vec3::Y,
                            KeyCode::KeyE => world.camera.position += camera_speed * Vec3::Y,
                            _ => (),
                        }
                        window.request_redraw();
                    }
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    } => match state {
                        ElementState::Pressed => lmb_pressed = true,
                        ElementState::Released => lmb_pressed = false,
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        if lmb_pressed {
                            if let Some(last_cursor) = last_cursor {
                                let x_change = -(position.x - last_cursor.x) as f32;
                                let y_change = (position.y - last_cursor.y) as f32;
                                world.camera.yaw += Rad32::new(x_change) * sensitivity;
                                world.camera.pitch += Rad32::new(y_change) * sensitivity;
                                world.camera.pitch = world
                                    .camera
                                    .pitch
                                    .clamp(-Rad32::QUARTER_TURN, Rad32::QUARTER_TURN);
                                window.request_redraw();
                            }
                            last_cursor = Some(position);
                        } else {
                            last_cursor = None;
                        }
                    }
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(_, y),
                        ..
                    } => {
                        world.camera.focal_length += y * 100000f32 * dt;
                        world.camera.focal_length = world.camera.focal_length.max(0f32);
                        window.request_redraw();
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }) {
        eprint!("Event loop error: {e:?}");
    }
}
