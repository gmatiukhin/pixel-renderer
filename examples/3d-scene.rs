use glam::Vec3;
use palette::{Srgb, Srgba};
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

use obj::{load_obj, Obj};
use std::fs::File;
use std::io::BufReader;

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

    let args: Vec<String> = std::env::args().collect();
    let input =
        BufReader::new(File::open(args[1].clone()).expect("Could not open file with given path"));
    let obj = ObjWrapper(load_obj(input).expect("Could not load obj file at the given path"));

    let mut world = World {
        camera: Camera {
            aperture: (35, 24),
            focal_length: 10f32,
            near: 0.1f32,
            far: 10f32,
            fit_strategy: FitStrategy::Overscan,
            position: Vec3::new(0f32, 0f32, 2f32),
            yaw: Rad32::new(-90f32),
            pitch: Rad32::new(0f32),
        },
        renderer: Rasterizer {
            output_width: width,
            output_height: height,
            show_wireframe: false,
            show_polygons: true,
        },
        objects: vec![Box::new(obj)],
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

struct ObjWrapper(Obj);

impl Mesh3D for ObjWrapper {
    fn vertices(&self) -> Vec<Vec3> {
        self.0
            .vertices
            .iter()
            .map(|v| {
                let p = v.position;
                Vec3::new(p[0], p[1], p[2])
            })
            .collect()
    }

    fn indices(&self) -> Vec<(usize, usize, usize)> {
        self.0
            .indices
            .chunks(3)
            .map(|e| (e[0] as usize, e[1] as usize, e[2] as usize))
            .collect()
    }

    fn attributes(&self) -> Vec<VertexAttribute> {
        let n = self.0.vertices.len();

        vec![
            VertexAttribute {
                color: Srgb::new(1f32, 1f32, 1f32),
            };
            n
        ]
    }
}
