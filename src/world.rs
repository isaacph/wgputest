use cgmath::{Vector2, Vector4};
use crate::graphics::Instance;
use crate::bounding_box::BoundingBox;

pub trait GameObject {
    fn get_instance(&self) -> Instance;  
}

pub struct Player {
    pub shape: BoundingBox,
    pub scale: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>, // in the future don't store any rendering info inside the world
}

impl GameObject for Player {
    fn get_instance(&self) -> Instance {
        return Instance {
            position: self.shape.center,
            scale: self.scale,
            color: self.color,
        };
    }
}

impl Player {
    pub fn new(shape: BoundingBox, scale: cgmath::Vector2<f32>, color: cgmath::Vector4<f32>) -> Self {
        Self {
            shape,
            scale,
            color,
        }
    }
}

pub struct StageObject {
    pub shape: BoundingBox,
    pub scale: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>, // in the future don't store any rendering info inside the world
}

impl GameObject for StageObject {
    fn get_instance(&self) -> Instance {
        return Instance {
            position: self.shape.center,
            scale: self.scale,
            color: self.color,
        };
    }
}

impl StageObject {
    pub fn new(shape: BoundingBox, scale: cgmath::Vector2<f32>, color: cgmath::Vector4<f32>) -> Self {
        Self {
            shape,
            scale,
            color,
        }
    }
}

pub struct World {
    pub objects: Vec<Box<dyn GameObject>>,
}

impl World {
    pub fn new() -> Self {
        

        // use cgmath::Vector2;
        // let objects = (0..10).into_iter().map(|i| {
        //     GameObject {
        //         position: Vector2::new((i as f32) * 0.8, (i as f32) * 0.8),
        //         scale: Vector2::new(1.5, 0.9),
        //         color: Vector4::new(
        //             if i % 2 == 0 { 0.0 } else { 1.0 },
        //             if i % 2 == 1 { 0.0 } else { 1.0 },
        //             0.0,
        //             1.0
        //         ),
        //     }
        // }).collect::<Vec<_>>();
        // Self {
        //     objects
        // }

    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        // match event {
        //     WindowEvent::KeyboardInput {
        //         input: KeyboardInput {
        //             state,
        //             virtual_keycode: Some(key), ..
        //         }, ..
        //     } if self.relevant_inputs.contains(&key) => {
        //         match state {
        //             ElementState::Pressed => self.inputs.insert(*key),
        //             ElementState::Released => self.inputs.remove(key),
        //         };
        //         true;
        //         match key {
                    
        //         }
        //     },

        //     _ => false,
        // }
    }

    pub fn update(&mut self, delta_time: f32) {
    }
}
