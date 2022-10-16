use cgmath::{Vector2, Zero, Point2, EuclideanSpace};
use graphics::RenderEngine;
use instant::Instant;
use std::collections::HashSet;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use world::World;

mod bounding_box;
mod camera;
mod graphics;
mod world;
pub mod util;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    // make webassembly work
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(800, 600));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) { // UPDATED!
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {},
                    // Reconfigure the surface if lsot
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            },
            _ => {}
        }
    });
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_engine: RenderEngine,

    camera: camera::Camera,
    camera_controller: camera::CameraController,

    last_frame: Instant,

    pub world: World,
    pub input_state: InputState,
    pub mouse_pos_view: Vector2<f32>,
}

pub struct InputState {
    pub key_down: HashSet<VirtualKeyCode>,
    pub key_pos_edge: HashSet<VirtualKeyCode>,
    pub key_neg_edge: HashSet<VirtualKeyCode>,
    pub mouse_pos_edge: HashSet<MouseButton>,
    pub mouse_position: Vector2<f32>,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // // load a texture - happy tree
        // let diffuse_bytes = include_bytes!("happy-tree.png");
        // let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        // let diffuse_texture =
        //     texture::Texture::from_image(&device, &queue, &diffuse_image, "happy-tree").unwrap();

        // // load a texture - keyboard
        // let diffuse_bytes_2 = include_bytes!("keyboard.jpg");
        // let diffuse_image_2 = image::load_from_memory(diffuse_bytes_2).unwrap();
        // let diffuse_texture_2 =
        //     texture::Texture::from_image(&device, &queue, &diffuse_image_2, "keyboard").unwrap();

        // camera
        let camera = camera::Camera::new(cgmath::Vector2::new(size.width, size.height), 10.0);
        let camera_controller = camera::CameraController::new(1.0);

        let render_engine = RenderEngine::init(&device, &queue, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_engine,
            camera,
            camera_controller,
            last_frame: Instant::now(),
            world: World::new(),
            input_state: InputState {
                key_down: HashSet::new(),
                key_pos_edge: HashSet::new(),
                key_neg_edge: HashSet::new(),
                mouse_pos_edge: HashSet::new(),
                mouse_position: Vector2::zero(),
            },
            mouse_pos_view: Vector2::zero(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.window_size = cgmath::Vector2::new(new_size.width, new_size.height);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        let relevant_inputs = {
            use VirtualKeyCode::*;
            vec![A, S, D, W, E, Space, LShift]
        };
        if !self.camera_controller.process_events(event) {
            match *event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode:
                        Some(key),
                        ..
                    },
                    ..
                } if relevant_inputs.contains(&key) => {
                    match state {
                        ElementState::Pressed => {
                            self.input_state.key_down.insert(key);
                            self.input_state.key_pos_edge.insert(key);
                        },
                        ElementState::Released => {
                            self.input_state.key_down.remove(&key);
                            self.input_state.key_neg_edge.insert(key);
                        },
                    };
                    true
                },
                WindowEvent::CursorMoved { position, .. } => {
                    let pos = Point2::new(position.x as f32, position.y as f32);
                    self.mouse_pos_view = pos.to_vec();
                    self.input_state.mouse_position = self.camera.view_to_world_pos(pos).to_vec();
                    true
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button,
                    ..
                } => {
                    self.input_state.mouse_pos_edge.insert(button);
                    true
                },
                _ => false,
            }
        } else {
            true
        }
    }

    fn update(&mut self) {
        // timing
        let frame = Instant::now();
        let delta_time = ((frame - self.last_frame).as_nanos() as f64 / 1000000000.0) as f32;
        self.last_frame = frame;

        self.world.update(delta_time, &self.input_state);

        // camera update
        self.camera_controller.update_camera(delta_time, &mut self.camera);
        
        // clear inputs
        self.input_state.key_pos_edge.clear();
        self.input_state.key_neg_edge.clear();
        self.input_state.mouse_pos_edge.clear();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_prereq = graphics::RenderPrereq {
            device: &mut self.device,
            queue: &mut self.queue,
            surface: &mut self.surface,
            camera: &self.camera
        };
        self.render_engine.render(render_prereq, &self.world)
    }
}
