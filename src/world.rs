use cgmath::Vector2;
use crate::bounding_box::BoundingBox;
use uuid::Uuid;
use std::collections::HashMap;
use player::Player;
use self::physics::{PhysicsObject, Physics};

// pub mod basic_enemy;
pub mod player;
pub mod physics;

pub trait IDObject {
    fn get_uuid(&self) -> Uuid;
}

pub trait GameObject: Physics + IDObject {
}

// pub struct GameObjectData {
//     // carryover info from preexisting GameObject class 
//     pub scale: Vector2<f32>,
//     pub color: Vector4<f32>, // in the future don't store any rendering info inside the world
// }
// 
// impl GameObjectData {
//     pub fn new_moving(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>, 
//             velocity: Vector2<f32>) -> Self {
//         Self {
//             bounding_box: BoundingBox::new(position, 1.0 * scale.x, 1.0 * scale.y),
//             velocity,
//             scale,
//             color,
//         }
//     }
//     
//     pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
//         Self {
//             bounding_box: BoundingBox::new(position, 1.0 * scale.x, 1.0 * scale.y),
//             velocity: Vector2::new(0.0, 0.0),
//             scale,
//             color,
//         }
//     }
// 
//     fn get_instance(&self) -> Instance {
//         return Instance {
//             position: self.bounding_box.center,
//             scale: self.scale,
//             color: self.color,
//         };
//     }
// }


pub struct Projectile {
    id: Uuid,
    physics: PhysicsObject
}


impl Projectile {
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>) -> Self {
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, scale.x, scale.y),
            velocity: Vector2::new(0.0, 0.0),
            can_move: true
        };
        Self {
            id: Uuid::new_v4(),
            physics,
        }
    }
}

impl GameObject for Projectile {
}

impl IDObject for Projectile {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for Projectile {
    fn get_physics(&self) -> Option<(Uuid, PhysicsObject)> {
        Some((self.id, self.physics.clone()))
    }

    fn resolve(&mut self, delta: Vector2<f32>, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(delta + resolve);
        delta + resolve
    }

    fn pre_physics(&mut self) {
    }
}

pub struct Stage {
    id: Uuid,
    pub physics: PhysicsObject
}


impl Stage {
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>) -> Self {
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, scale.x, scale.y),
            velocity: Vector2::new(0.0, 0.0),
            can_move: false
        };
        Self {
            id: Uuid::new_v4(),
            physics,
        }
    }
}

impl GameObject for Stage {
}

impl IDObject for Stage {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for Stage {
    fn get_physics(&self) -> Option<(Uuid, PhysicsObject)> {
        Some((self.id, self.physics.clone()))
    }

    fn resolve(&mut self, delta: Vector2<f32>, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(delta + resolve);
        delta + resolve
    }

    fn pre_physics(&mut self) {
    }
}

pub struct World {
    // pub objects: HashMap<Uuid, Box<dyn GameObject>>,
    pub player: Player,
    pub stage: Vec<Stage>,

    pub debug_objects: Vec<crate::graphics::ResolveInstance>,
}

impl World {
    pub fn new() -> Self {
        let player = Player::new(
            Vector2::new(0.0, 0.0)
        );

        let stage_left = Stage::new(
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.25, 1.0),
        );

        let stage_right = Stage::new(
            Vector2::new(1.0, 0.0),
            Vector2::new(0.25, 1.0),
        );

        let stage_top = Stage::new(
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 0.25),
        );

        let stage_down = Stage::new(
            Vector2::new(0.0, 5.0),
            Vector2::new(5.0, 0.25),
        );
        let stage = vec![stage_left, stage_right, stage_top, stage_down];

        // let objects: Vec<Box<dyn GameObject>> = vec![player, stage_left, stage_right, stage_top, stage_down];

        // let objects = objects.into_iter().map(|obj| (obj.get_uuid(), obj)).collect();
        Self {
            player,
            stage,
            debug_objects: vec![]
        }
    }

    // don't we need a thing to tell it how much to change?
    pub fn update(&mut self, delta_time: f32, input_state: &crate::InputState) {
        // let move_vec = {
        //     use VirtualKeyCode::*;
        //     Vector2::new(
        //         (input_state.key_down.contains(&D) as i32 - input_state.key_down.contains(&A) as i32) as f32,
        //         (input_state.key_down.contains(&S) as i32 - input_state.key_down.contains(&W) as i32) as f32,
        //     )
        // };
        // // move player by move vec
        // self.player.physics.velocity = move_vec;
        self.player.update(delta_time, input_state);

        // update
        // for (id, object) in self.objects {
        //     
        // }

        self.physics(delta_time);
    }

    fn physics(&mut self, delta_time: f32) {
        // gather all who want to be physic'd
        let mut to_physics_on: Vec<&mut dyn Physics> = vec![
            &mut self.player
        ];
        to_physics_on.extend(
            self.stage.iter_mut().map(|stage| stage as &mut dyn Physics)
        );

        // construct simulation data
        let physics_objects: HashMap<Uuid, PhysicsObject> = to_physics_on.iter()
            .map(|game_obj| game_obj.get_physics()).flatten().collect();

        // restructure to turn into callbackable objects
        let mut id_objs: HashMap<Uuid, &mut dyn Physics> = to_physics_on.into_iter()
            .map(|obj| (obj.get_uuid(), obj)).collect();
        
        // pre-physics step
        id_objs.values_mut().for_each(|o| o.pre_physics());

        // simulate them
        physics::simulate(delta_time, physics_objects, |id, delta, resolve, p_obj| {
            id_objs.get_mut(&id).map(|obj| {
                let obj: &mut dyn Physics = *obj;
                p_obj.bounding_box.add(obj.resolve(delta, resolve));
            });
        });
    }
}

