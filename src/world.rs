use std::ops::Add;

use cgmath::{Vector2, Vector4, prelude::InnerSpace};
use crate::graphics::textured::Instance;
use crate::bounding_box::BoundingBox;
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use uuid::{uuid, Uuid};
use std::collections::HashMap;

// pub mod basic_enemy;

pub trait IDObject {
    fn get_uuid(&self) -> Uuid;
}

pub trait Physics: IDObject {
    fn get_physics(&self) -> Option<(Uuid, PhysicsObject)>;
    fn resolve(&mut self, resolve: Vector2<f32>) -> Vector2<f32>;
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

pub struct Player {
    id: Uuid,
    pub physics: PhysicsObject
}


impl Player {
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

impl GameObject for Player {
}

impl IDObject for Player {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for Player {
    fn get_physics(&self) -> Option<(Uuid, PhysicsObject)> {
        Some((self.id, self.physics.clone()))
    }

    fn resolve(&mut self, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(resolve);
        resolve
    }
}


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

    fn resolve(&mut self, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(resolve);
        resolve
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

    fn resolve(&mut self, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(resolve);
        resolve
    }
}

pub struct World {
    // pub objects: HashMap<Uuid, Box<dyn GameObject>>,
    pub player: Player,
    pub stage: Vec<Stage>,

    pub debug_objects: Vec<Instance>,
}

impl World {
    pub fn new() -> Self {
        let black_color: Vector4<f32> = Vector4::new(0.0, 0.0, 0.0, 1.0);
        
        let player = Player::new(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 1.0),
        );
        let player_id = player.get_uuid();

        let stage_left = Stage::new(
            Vector2::new(-1.0, 0.0),
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
            Vector2::new(0.0, -1.0),
            Vector2::new(1.0, 0.25),
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
        let move_vec = {
            use VirtualKeyCode::*;
            Vector2::new(
                (input_state.key_down.contains(&D) as i32 - input_state.key_down.contains(&A) as i32) as f32,
                (input_state.key_down.contains(&S) as i32 - input_state.key_down.contains(&W) as i32) as f32,
            )
        } * (input_state.key_down.contains(&VirtualKeyCode::Space) as i32 as f32 * 0.6 + 0.02);

        // move player by move vec
        self.player.physics.velocity = move_vec;

        // update
        // for (id, object) in self.objects {
        //     
        // }

        self.physics(delta_time);
    }

    fn physics(&mut self, delta_time: f32) {
        // gather all who want to be physic'd
        let x = &mut self.player;
        let mut to_physics_on: Vec<&mut dyn Physics> = vec![
            x
        ];
        to_physics_on.extend(self.stage.iter_mut().map(|stage| {
            let x: &mut dyn Physics = stage;
            x
        }));

        // grab simulation data
        let physics_objects: HashMap<Uuid, PhysicsObject> = to_physics_on.iter()
            .map(|game_obj| game_obj.get_physics()).flatten().collect();

        // restructure so we can find callbacks
        let mut id_objs: HashMap<Uuid, &mut dyn Physics> = to_physics_on.into_iter()
            .map(|obj| (obj.get_uuid(), obj)).collect();

        // simulate them
        self.debug_objects = simulate(delta_time, physics_objects, |id, resolve, p_obj| {
            id_objs.get_mut(&id).map(|obj| {
                let obj: &mut dyn Physics = *obj;
                p_obj.bounding_box.add(obj.resolve(resolve));
            });
        });
    }
}

#[derive(Clone)]
pub struct PhysicsObject {
    pub bounding_box: BoundingBox,
    pub velocity: Vector2<f32>,
    pub can_move: bool,
}

type PhysicsID = Uuid;
pub struct PhysicsData {
    pub objects: HashMap<PhysicsID, PhysicsObject>,
}

impl PhysicsData {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}
pub fn simulate
    <F: FnMut(PhysicsID, Vector2<f32>, &mut PhysicsObject)>
    (delta_time: f32, mut objects: HashMap<PhysicsID, PhysicsObject>, mut resolve: F) -> Vec<Instance> {
    //  for each moveable object
    //      move object in x direction
    //          check collisions in x
    //          respond to collisions in x
    //      move object in y direction
    //          check collisions in y
    //          respond to collisions in y
    let mut debug_objects = vec![];

    let obj_ids: Vec<Uuid> = objects.keys().cloned().collect();
    for id in obj_ids {
        let obj = objects.get(&id).unwrap();
        if obj.can_move {
            let delta = obj.velocity * delta_time;
            let box_a = obj.bounding_box.clone();
            // let mut total_resolve = Vector2::new(0.0, 0.0);
            let mut total_resolve = delta;

            // finds the number of overlaps of one bounding box against the self

            let find_overlaps = |objects: &HashMap<Uuid, PhysicsObject>, box_a: &BoundingBox, box_a_id: Uuid| {
                objects.iter().fold(0, |count, (other_id, other)| {
                    if *other_id != box_a_id {
                        let box_b = other.bounding_box.clone();
                        if box_a.does_intersect(&box_b) {
                            return count + 1
                        }
                    }
                    count
                })
            };

            for delta in [
                Vector2::new(delta.x, 0.0),
                Vector2::new(0.0, delta.y),
            ] {
                let mut box_a = box_a.clone();
                box_a.add(total_resolve);

                // move in delta direction
                box_a.add(delta);

                // get starting overlaps
                let starting_overlaps = find_overlaps(&objects, &box_a, id);

                // find best way to resolve collisions
                // this method adds the current position
                // let mut best_resolve: Vector2<f32> = -delta;
                // let mut best_resolve_len_sq = delta.magnitude2();
                // let mut best_resolve_overlaps = starting_overlaps;
                let mut best_resolve: Vector2<f32> = Vector2::new(0.0, 0.0);
                let mut best_resolve_len_sq = delta.magnitude2();
                let mut best_resolve_overlaps = starting_overlaps;
                for (other_id, other) in &objects {
                    if *other_id != id {
                        // got a non-me other
                        let box_b = other.bounding_box.clone();
                        let resolve_options = box_a.resolve_options(&box_b);
                        
                        for resolve_option in resolve_options {
                            // determine how good the resolve option is
                            let box_a_resolved = {
                                let mut box_a = box_a.clone();
                                box_a.add(resolve_option);
                                box_a
                            };
                            let new_overlaps = find_overlaps(&objects, &box_a_resolved, id);
                            let new_len_sq =
                                resolve_option.x * resolve_option.x +
                                resolve_option.y * resolve_option.y;
                            
                            // if it's better than best then use it
                            if new_overlaps < best_resolve_overlaps ||
                                    (new_overlaps == best_resolve_overlaps &&
                                    new_len_sq < best_resolve_len_sq) {
                                best_resolve = resolve_option;
                                best_resolve_len_sq = new_len_sq;
                                best_resolve_overlaps = new_overlaps;

                                debug_objects.push(Instance {
                                    position: box_a_resolved.center,
                                    scale: box_a.get_scale(),
                                    color: Vector4::new(1.0, 0.0, 0.0, 0.2),
                                });
                            } else {
                                debug_objects.push(Instance {
                                    position: box_a_resolved.center,
                                    scale: box_a.get_scale(),
                                    color: Vector4::new(1.0, 1.0, 0.0, 0.02),
                                });
                            }
                        }
                    }
                }
                best_resolve = Vector2::new(0.0, 0.0);

                total_resolve += delta + best_resolve;
            }

            let obj = objects.get_mut(&id).unwrap();
            resolve(id, total_resolve, obj);
        }
    }
    debug_objects
}
