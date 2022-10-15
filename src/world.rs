use std::ops::Add;

use cgmath::{Vector2, Vector4, prelude::InnerSpace};
use crate::graphics::textured::Instance;
use crate::bounding_box::BoundingBox;
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use uuid::{uuid, Uuid};
use std::collections::HashMap;

pub trait IDObject {
    fn get_uuid(&self) -> Uuid;
}

pub trait Physics: IDObject {
    fn get_bounding_box(&self) -> BoundingBox;
    fn get_velocity(&self) -> Vector2<f32>;
    fn respond_to_resolution(&mut self, delta_position: Vector2<f32>, objects: &Vec<Box<dyn GameObject>>);
    fn add_velocity(&mut self, delta_velocity: Vector2<f32>);
    fn add_position(&mut self, delta_position: Vector2<f32>);
    fn can_move(&self) -> bool;
}

pub trait GameObject: Physics + IDObject {
    fn get_instance(&self) -> Instance;
}

pub struct GameObjectData {
    pub bounding_box: BoundingBox,
    pub velocity: Vector2<f32>,
    
    // carryover info from preexisting GameObject class 
    pub scale: Vector2<f32>,
    pub color: Vector4<f32>, // in the future don't store any rendering info inside the world
}

impl GameObjectData {
    pub fn new_moving(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>, 
            velocity: Vector2<f32>) -> Self {
        Self {
            bounding_box: BoundingBox::new(position, 1.0 * scale.x, 1.0 * scale.y),
            velocity,
            scale,
            color,
        }
    }
    
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
        Self {
            bounding_box: BoundingBox::new(position, 1.0 * scale.x, 1.0 * scale.y),
            velocity: Vector2::new(0.0, 0.0),
            scale,
            color,
        }
    }

    fn get_instance(&self) -> Instance {
        return Instance {
            position: self.bounding_box.center,
            scale: self.scale,
            color: self.color,
        };
    }
}

pub struct Player {
    id: Uuid,
    data: GameObjectData
}


impl Player {
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
        Self {
            data: GameObjectData::new(position, scale, color),
            id: Uuid::new_v4(),
        }
    }
}

impl GameObject for Player {
    fn get_instance(&self) -> Instance {
        return self.data.get_instance();
    }
}

impl IDObject for Player {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for Player {
    fn get_bounding_box(&self) -> BoundingBox {
        return self.data.bounding_box.clone();
    }

    fn get_velocity(&self) -> Vector2<f32> {
        return self.data.velocity;
    }

    fn respond_to_resolution(&mut self, delta_position: Vector2<f32>, _objects: &Vec<Box<dyn GameObject>>) {
        // isaac halp
        // easy impl:
        self.add_position(delta_position);
    }

    fn add_velocity(&mut self, delta_velocity: Vector2<f32>) {
        self.data.velocity += delta_velocity;
    }

    fn add_position(&mut self, delta_position: Vector2<f32>) {
        self.data.velocity += delta_position;
    }

    fn can_move(&self) -> bool {
        return true;
    }
}

pub struct Enemy {
    id: Uuid,
    data: GameObjectData
}

impl Enemy {
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
        Self {
            data: GameObjectData::new(position, scale, color),
            id: Uuid::new_v4()
        }
    }
}

impl GameObject for Enemy {
    fn get_instance(&self) -> Instance {
        return self.data.get_instance();
    }
}

impl IDObject for Enemy {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for Enemy {
    fn get_bounding_box(&self) -> BoundingBox {
        return self.data.bounding_box.clone();
    }

    fn get_velocity(&self) -> Vector2<f32> {
        return self.data.velocity;
    }

    fn respond_to_resolution(&mut self, delta_position: Vector2<f32>, _objects: &Vec<Box<dyn GameObject>>) {
        // isaac halp
        // easy impl:
        self.add_position(delta_position);
    }

    fn add_velocity(&mut self, delta_velocity: Vector2<f32>) {
        self.data.velocity += delta_velocity;
    }

    fn add_position(&mut self, delta_position: Vector2<f32>) {
        self.data.velocity += delta_position;
    }

    fn can_move(&self) -> bool {
        return true;
    }
}

pub struct Projectile {
    id: Uuid,
    data: GameObjectData
}

impl Projectile {
    // projectiles SHOULD be initialized with velocity
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>, 
            velocity: Vector2<f32>) -> Self {
        Self {
            data: GameObjectData::new_moving(position, scale, color, velocity),
            id: Uuid::new_v4(),
        }
    }

}

impl GameObject for Projectile {
    fn get_instance(&self) -> Instance {
        return self.data.get_instance();
    }
}

impl IDObject for Projectile {
    fn get_uuid(&self) -> Uuid {
        self.id
    }
}

impl Physics for Projectile {
    fn get_bounding_box(&self) -> BoundingBox {
        return self.data.bounding_box.clone();
    }

    fn get_velocity(&self) -> Vector2<f32> {
        return self.data.velocity;
    }

    fn respond_to_resolution(&mut self, delta_position: Vector2<f32>, objects: &Vec<Box<dyn GameObject>>) {
        // isaac halp
        // easy impl:
        self.add_position(delta_position);
    }

    fn add_velocity(&mut self, delta_velocity: Vector2<f32>) {
        self.data.velocity += delta_velocity;
    }

    fn add_position(&mut self, delta_position: Vector2<f32>) {
        self.data.velocity += delta_position;
    }

    fn can_move(&self) -> bool {
        // projectile should just disappear on hit, so this doesn't really matter either way
        return true;
    }
}

pub struct Stage {
    id: Uuid,
    data: GameObjectData
}

impl Stage {
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
        Self {
            data: GameObjectData::new(position, scale, color),
            id: Uuid::new_v4(),
        }
    }
}

impl GameObject for Stage {
    fn get_instance(&self) -> Instance {
        return self.data.get_instance();
    }
}

impl IDObject for Stage {
    fn get_uuid(&self) -> Uuid {
        self.id
    }
}

impl Physics for Stage {
    fn get_bounding_box(&self) -> BoundingBox {
        return self.data.bounding_box.clone();
    }

    fn get_velocity(&self) -> Vector2<f32> {
        return self.data.velocity;
    }

    fn respond_to_resolution(&mut self, delta_position: Vector2<f32>, _objects: &Vec<Box<dyn GameObject>>) {
        // isaac halp
        // easy impl:
        self.add_position(delta_position);
    }

    fn add_velocity(&mut self, delta_velocity: Vector2<f32>) {
        self.data.velocity += delta_velocity;
    }

    fn add_position(&mut self, delta_position: Vector2<f32>) {
        self.data.velocity += delta_position;
    }

    fn can_move(&self) -> bool {
        return false;
    }
}

pub struct World {
    pub objects: HashMap<Uuid, Box<dyn GameObject>>,
    pub player_id: Uuid,
}

impl World {
    pub fn new() -> Self {
        let black_color: Vector4<f32> = Vector4::new(0.0, 0.0, 0.0, 0.0);
        
        let player = Box::new(Player::new(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 1.0),
            black_color
        ));
        let player_id = player.get_uuid();

        let stage_left = Box::new(Stage::new(
            Vector2::new(-1.0, 0.0),
            Vector2::new(0.25, 1.0),
            black_color
        ));

        let stage_right = Box::new(Stage::new(
            Vector2::new(1.0, 0.0),
            Vector2::new(0.25, 1.0),
            black_color
        ));

        let stage_top = Box::new(Stage::new(
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 0.25),
            black_color
        ));

        let stage_down = Box::new(Stage::new(
            Vector2::new(0.0, -1.0),
            Vector2::new(1.0, 0.25),
            black_color
        ));

        let objects: Vec<Box<dyn GameObject>> = vec![player, stage_left, stage_right, stage_top, stage_down];

        let objects = objects.into_iter().map(|obj| (obj.get_uuid(), obj)).collect();
        Self {
            objects,
            player_id,
        }
    }

    // don't we need a thing to tell it how much to change?
    pub fn update(&mut self, delta_time: f32, input_state: &crate::InputState) {
        let move_vec = {
            use VirtualKeyCode::*;
            Vector2::new(
                (input_state.key_down.contains(&D) as i32 - input_state.key_down.contains(&A) as i32) as f32,
                (input_state.key_down.contains(&W) as i32 - input_state.key_down.contains(&S) as i32) as f32,
            )
        };

        // move player by move vec
        self.objects.get_mut(&self.player_id).map(|player| {
            let velocity = player.get_velocity();
            player.add_velocity(move_vec * delta_time - velocity);
        });

        // update
        // for (id, object) in self.objects {
        //     
        // }

        // physics step
        physics_step(self, delta_time);
    }
}

pub fn physics_step(world: &mut World, delta_time: f32) {
    //  for each moveable object
    //      move object in x direction
    //          check collisions in x
    //          respond to collisions in x
    //      move object in y direction
    //          check collisions in y
    //          respond to collisions in y

    let obj_ids: Vec<Uuid> = world.objects.keys().cloned().collect();
    println!("1");
    for id in obj_ids {
        let obj = world.objects.get(&id).unwrap();
        let delta = obj.get_velocity() * delta_time;
        let box_a = obj.get_bounding_box();
        let mut total_resolve = Vector2::new(0.0, 0.0);

        // finds the number of overlaps of one bounding box against the world
        let find_overlaps = |box_a: &BoundingBox, box_a_id: Uuid| {
            world.objects.values().fold(0, |count, other| {
                if other.get_uuid() != box_a_id {
                    let box_b = other.get_bounding_box();
                    if box_a.does_intersect(&box_b) {
                        return count + 1
                    }
                }
                count
            })
        };

        println!("2");
        for delta in [
            Vector2::new(delta.x, 0.0),
            Vector2::new(0.0, delta.y),
        ] {
            let mut box_a = box_a.clone();
            box_a.add(total_resolve);

            // get starting overlaps
            let starting_overlaps = find_overlaps(&box_a, id);
            println!("3");

            // move in delta direction
            box_a.add(delta);

            // find best way to resolve collisions
            let mut best_resolve: Vector2<f32> = -delta;
            let mut best_resolve_len_sq = delta.magnitude2();
            let mut best_resolve_overlaps = starting_overlaps;
            for other in world.objects.values() {
                if other.get_uuid() != id {
                    // got a non-me other
                    let box_b = other.get_bounding_box();
                    let resolve_options = box_a.resolve_options(&box_b);
                    
                    println!("5");
                    for resolve_option in resolve_options {
                        // determine how good the resolve option is
                        let box_a_resolved = {
                            let mut box_a = box_a.clone();
                            box_a.add(resolve_option);
                            box_a
                        };
                        let new_overlaps = find_overlaps(&box_a_resolved, id);
                        let new_len_sq =
                            resolve_option.x * resolve_option.x +
                            resolve_option.y * resolve_option.y;
                        
                        println!("6");
                        // if it's better than best then use it
                        if new_overlaps < best_resolve_overlaps ||
                                (new_overlaps == best_resolve_overlaps &&
                                new_len_sq < best_resolve_len_sq) {
                            best_resolve = resolve_option;
                            best_resolve_len_sq = new_len_sq;
                            best_resolve_overlaps = new_overlaps;
                        }
                    }
                }
            }

            total_resolve += delta + best_resolve;
        }

        let obj = world.objects.get_mut(&id).unwrap();
        obj.respond_to_resolution(total_resolve, &vec![]);
    }
}
