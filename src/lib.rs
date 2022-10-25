#![windows_subsystem = "windows"]
use audio::Audio;
use cgmath::{Vector2, Zero, Point2, EuclideanSpace, Vector4};
use chatbox::Chatbox;
use graphics::{RenderEngine, text::BaseFontInfoContainer};
use instant::Instant;
use std::{collections::HashSet, rc::Rc};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use world::World;

use crate::{world::stage, graphics::ResolveInstance};

mod bounding_box;
mod camera;
mod graphics;
mod world;
pub mod util;
pub mod chatbox;
pub mod audio;

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
    let window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());

    #[cfg(target_arch = "wasm32")]
    let c_window = window.clone();
    #[cfg(target_arch = "wasm32")]
    let resize_closure = wasm_bindgen::prelude::Closure::<dyn FnMut()>::new(move || {
        web_sys::window()
            .and_then(|win| Some((win.document().unwrap(), win.device_pixel_ratio())))
            .and_then(|(doc, ratio)| {
                use winit::dpi::PhysicalSize;
                let dst = doc.get_element_by_id("wasm-example")?;
                let width = (dst.client_width() as f64 * ratio) as i32;
                let height = (dst.client_height() as f64 * ratio) as i32;
                c_window.set_inner_size(PhysicalSize::new(width, height));
                Some(())
            })
            .expect("Couldn't resize");
    });


    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;
        // window.set_inner_size(PhysicalSize::new(800, 600));
        
        use winit::platform::web::WindowExtWebSys;
        let window = window.clone();
        web_sys::window()
            .and_then(|win| {
                let window = window.clone();
                win.set_onresize(Some(resize_closure.as_ref().unchecked_ref()));
                Some((win.document().unwrap(), win.device_pixel_ratio()))
            })
            .and_then(|(doc, ratio)| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let width = (dst.client_width() as f64 * ratio) as i32;
                let height = (dst.client_height() as f64 * ratio) as i32;
                window.set_inner_size(PhysicalSize::new(width, height));
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
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
                match state.update() {
                    true => {
                        *control_flow = ControlFlow::Exit
                    },
                    false => (),
                }
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusMode {
    Default, Chatbox
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameState {
    Game, Editor
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

    pub chatbox: Chatbox,
    pub focus_mode: FocusMode,
    pub game_state: GameState,

    pub audio: Audio,
}

pub struct InputState {
    pub key_down: HashSet<VirtualKeyCode>,
    pub key_pos_edge: HashSet<VirtualKeyCode>,
    pub key_neg_edge: HashSet<VirtualKeyCode>,
    pub mouse_pos_edge: HashSet<MouseButton>,
    pub mouse_position: Vector2<f32>,
    pub commands: Vec<String>,
    pub edit: bool,
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
        let camera = camera::Camera::new(cgmath::Vector2::new(size.width, size.height), 20.0);
        let camera_controller = camera::CameraController::new(1.0);

        let render_engine = RenderEngine::init(&device, &queue, &config);
        let chatbox = Chatbox::new(render_engine.font.get_metrics_info(), 7, 38.0, 7, 800.0);

        let mut audio = Audio::new();
        #[cfg(not(target_arch = "wasm32"))]
        audio.init_audio();
        audio.play(audio::Song::Church);

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
                commands: vec![],
                edit: true,
            },
            mouse_pos_view: Vector2::zero(),
            chatbox,
            focus_mode: FocusMode::Default,
            game_state: GameState::Game,
            audio,
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
        if self.focus_mode == FocusMode::Chatbox {
            match *event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode:
                        Some(key),
                        ..
                    },
                    ..
                } => {
                    match key {
                        VirtualKeyCode::Escape => {
                            self.focus_mode = FocusMode::Default;
                            self.chatbox.set_typing_flicker(false);
                        },
                        VirtualKeyCode::Return => {
                            if self.chatbox.get_typing().is_empty() {
                                self.focus_mode = FocusMode::Default;
                                self.chatbox.set_typing_flicker(false);
                            } else {
                                let typing = self.chatbox.get_typing().clone();
                                self.chatbox.println(&typing);
                                self.chatbox.erase_typing();
                                self.focus_mode = FocusMode::Default;
                                self.chatbox.set_typing_flicker(false);
                                self.input_state.commands.push(typing);
                            }
                        },
                        _ => {
                        }
                    }
                    return true
                },
                WindowEvent::ReceivedCharacter(c) => {
                    if c == '\x08' { // backspace
                        self.chatbox.remove_typing(1);
                    } else if c == '\n' || c == '\r' || c.to_string().as_bytes()[0] == 13 {
                        // ahhhhhhhhhhh why can't we capture new lines?
                    } else {
                        self.chatbox.add_typing(c);
                    }
                },
                _ => ()
            };
        }
        let relevant_inputs = {
            use VirtualKeyCode::*;
            vec![Key1, Key2, A, S, D, W, E, Space, LShift]
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
                } => match key {
                    VirtualKeyCode::Return => {
                        self.focus_mode = FocusMode::Chatbox;
                        self.chatbox.set_typing_flicker(true);
                        true
                    },
                    VirtualKeyCode::R => {
                        self.world = World::new();
                        true
                    },
                    _ => {
                        if relevant_inputs.contains(&key) {
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
                        } else { false }
                    }
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
                    self.audio.init_audio();
                    self.input_state.mouse_pos_edge.insert(button);
                    true
                },
                _ => false,
            }
        } else {
            true
        }
    }

    fn update(&mut self) -> bool {
        if self.input_state.commands.iter().map(|command| {
            match command.split(' ').collect::<Vec<_>>()[..] {
                ["exit"] => return true,
                ["edit"] => self.game_state = GameState::Editor,
                ["game"] => self.game_state = GameState::Game,
                _ => self.chatbox.println("Unknown command"),
            }
            return false;
        }).find(|x| *x).is_some() {
            return true
        }
        self.input_state.commands.clear();

        // timing
        let frame = Instant::now();
        let delta_time = ((frame - self.last_frame).as_nanos() as f64 / 1000000000.0) as f32;
        self.last_frame = frame;

        self.audio.update(delta_time);

        if self.game_state == GameState::Editor {
            // place blocks
            use stage::TileType::*;
            let pos = self.input_state.mouse_position;
            let rounded = Vector2::new((pos.x).floor() as i32, (pos.y).floor() as i32);
            if self.input_state.mouse_pos_edge.contains(&MouseButton::Left) {
                self.world.stage.values_mut().next().map(|stage| stage.set_tile(&rounded, Some(Dirt)));
            }
            if self.input_state.mouse_pos_edge.contains(&MouseButton::Right) {
                self.world.stage.values_mut().next().map(|stage| stage.set_tile(&rounded, None));
            }
            self.world.debug_objects = vec![
                ResolveInstance {
                    overlaps: 0,
                    color: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    position: Vector2::new(rounded.x as f32, rounded.y as f32) + Vector2::new(0.5, 0.5),
                    scale: Vector2::new(1.0, 1.0),
                }
            ];
        } else {
            // shoot stuff, implemented in world update
        }

        if self.game_state == GameState::Game {
            self.world.update(delta_time, &self.input_state);
        }

        // camera update
        self.camera_controller.update_camera(delta_time, &mut self.camera);

        self.chatbox.update(delta_time);
        
        // clear inputs
        self.input_state.key_pos_edge.clear();
        self.input_state.key_neg_edge.clear();
        self.input_state.mouse_pos_edge.clear();
        false
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_prereq = graphics::RenderPrereq {
            device: &mut self.device,
            queue: &mut self.queue,
            surface: &mut self.surface,
            camera: &self.camera
        };
        self.render_engine.render(render_prereq, &self.chatbox, &self.world)
    }
}
