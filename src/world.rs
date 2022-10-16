use cgmath::{Vector2, Vector4, InnerSpace};
use winit::event::MouseButton;
use crate::{bounding_box::BoundingBox, graphics::ResolveInstance, chatbox::Chatbox};
use uuid::Uuid;
use std::collections::HashMap;
use player::Player;
use self::{physics::{PhysicsObject, Physics}, stage::{Stage, TileType}, basic_enemy::BasicEnemy, jumping_enemy::JumpingEnemy, projectile::{Projectile, ProjectileType}};

pub mod jumping_enemy;
pub mod basic_enemy;
pub mod player;
pub mod physics;
pub mod stage;
pub mod projectile;

pub trait IDObject {
    fn get_uuid(&self) -> Uuid;
}

pub trait GameObject: Physics + IDObject {
}

pub struct World {
    // pub objects: HashMap<Uuid, Box<dyn GameObject>>,
    pub time_towards_next_spawn1: f32,
    pub time_towards_next_spawn2: f32,
    pub player: Player,
    pub basic_enemies: Vec<BasicEnemy>,
    pub jumping_enemies: Vec<JumpingEnemy>,
    pub stage: HashMap<Uuid, Stage>,
    pub projectiles: Vec<Projectile>,

    pub debug_objects: Vec<crate::graphics::ResolveInstance>,
}

pub enum GameStateChange {
    PlayerLose
}

impl World {
    const SPAWN_TIME1: f32 = 5.0;
    const SPAWN_TIME2: f32 = 8.0;

    pub fn new() -> Self {
        let player = Player::new(
            Vector2::new(-2.0, 2.0)
        );
        let basic_enemy = BasicEnemy::new(
            Vector2::new(2.0, 2.0)
        );
        let jumping_enemy = JumpingEnemy::new(
            Vector2::new(3.0, 2.0)
        );

        let mut stage = HashMap::new();
        stage.insert(Uuid::new_v4(), Stage::new());
        stage.values_mut().for_each(|stage| {
            stage.set_tile(&Vector2::new(4, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(3, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(2, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(1, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(0, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(0, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(-1, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(-2, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(-3, 10), Some(TileType::Dirt));
            stage.set_tile(&Vector2::new(-4, 10), Some(TileType::Dirt));
        });
        Self {
            time_towards_next_spawn1: 0.0, 
            time_towards_next_spawn2: 0.0, 
            player,
            basic_enemies: vec![basic_enemy],
            jumping_enemies: vec![jumping_enemy],
            stage,
            debug_objects: vec![],
            projectiles: vec![],
        }
    }

    // don't we need a thing to tell it how much to change?
    pub fn update(&mut self, delta_time: f32, input_state: &crate::InputState) {
        // increment time towards next spawn, spawn if appropriate
        self.time_towards_next_spawn1 += delta_time;
        if self.time_towards_next_spawn1 >= World::SPAWN_TIME1 {
            let basic_enemy = BasicEnemy::new(
                Vector2::new(2.0, 2.0)
            );
            self.basic_enemies.push(basic_enemy);

            self.time_towards_next_spawn1 = 0.0;
        }

        self.time_towards_next_spawn2 += delta_time;
        if self.time_towards_next_spawn2 >= World::SPAWN_TIME2 {
            let jumping_enemy = JumpingEnemy::new(
                Vector2::new(4.0, 2.0)
            );
            self.jumping_enemies.push(jumping_enemy);

            self.time_towards_next_spawn2 = 0.0;
        }


        // fire projectiles
        if self.player.alive && input_state.mouse_pos_edge.contains(&MouseButton::Left) {
            let mouse_pos = input_state.mouse_position;
            let dir = mouse_pos - self.player.physics.bounding_box.center;
            if dir.magnitude2() != 0.0 {
                let vel = dir.normalize() * 10.0;
                self.projectiles.push(
                    Projectile::new(self.player.physics.bounding_box.center,
                        // originally Basic projectile
                                    vel, projectile::ProjectileType::all()[self.player.current_projectile], physics::PhysObjType::Enemy));
            }
        }

        // update projectiles
        let mut to_destroy = vec![];
        for i in 0..self.projectiles.len() {
            let proj = &mut self.projectiles[i];
            proj.update(delta_time);
            if !proj.alive {
                to_destroy.push(i);
            }
        }
        for i in to_destroy.into_iter().rev() {
            self.projectiles.remove(i);
        }

        // update enemies
        let mut to_destroy = vec![];
        for i in 0..self.basic_enemies.len() {
            let obj = &mut self.basic_enemies[i];
            obj.update(delta_time, &mut self.player);
            if !obj.alive {
                to_destroy.push(i);
            }
        }
        for i in to_destroy.into_iter().rev() {
            self.basic_enemies.remove(i);
        }

        let mut to_destroy = vec![];
        for i in 0..self.jumping_enemies.len() {
            let obj = &mut self.jumping_enemies[i];
            obj.update(delta_time, &mut self.player);
            if !obj.alive {
                to_destroy.push(i);
            }
        }
        for i in to_destroy.into_iter().rev() {
            self.jumping_enemies.remove(i);
        }

        // let move_vec = {
        //     use VirtualKeyCode::*;
        //     Vector2::new(
        //         (input_state.key_down.contains(&D) as i32 - input_state.key_down.contains(&A) as i32) as f32,
        //         (input_state.key_down.contains(&S) as i32 - input_state.key_down.contains(&W) as i32) as f32,
        //     )
        // };
        // // move player by move vec
        // self.player.physics.velocity = move_vec;
        if self.player.alive {
            self.player.update(delta_time, input_state);
        }

        self.physics(delta_time);
    }

    fn physics(&mut self, delta_time: f32) {
        // gather all who want to be physic'd
        let mut to_physics_on: Vec<&mut dyn Physics> = vec![
            &mut self.player,
        ];

        // these two blocks look scuffed, i don't have time to make them elegant
        // add stage
        let mut temp_physics: Vec<_> = self.stage.iter()
            .map(|(_, stage)|
                 stage.get_physics()
                 .into_iter())
            .flatten()
            .collect();
        to_physics_on.extend(
            temp_physics
            .iter_mut()
            .map(|p| p as &mut dyn Physics)
        );
        // add projectiles
        to_physics_on.extend(
            self.projectiles
            .iter_mut()
            .map(|p| p as &mut dyn Physics)
        );
        // add enemies
        to_physics_on.extend(
            self.basic_enemies
            .iter_mut()
            .map(|p| p as &mut dyn Physics)
        );
        to_physics_on.extend(
            self.jumping_enemies
            .iter_mut()
            .map(|p| p as &mut dyn Physics)
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
        physics::simulate(delta_time, physics_objects, |id, delta, resolve, p_obj, types| {
            id_objs.get_mut(&id).map(|obj| {
                let obj: &mut dyn Physics = *obj;
                p_obj.bounding_box.add(obj.resolve(id, delta, resolve, types));
            });
        });
    }
}

