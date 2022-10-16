use cgmath::{Vector2, Vector4};
use winit::event::MouseButton;
use crate::{bounding_box::BoundingBox, graphics::ResolveInstance, chatbox::Chatbox};
use uuid::Uuid;
use std::collections::HashMap;
use player::Player;
use self::{physics::{PhysicsObject, Physics}, stage::Stage, basic_enemy::BasicEnemy};

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
    pub player: Player,
    pub basic_enemy: BasicEnemy,
    pub stage: HashMap<Uuid, Stage>,

    pub debug_objects: Vec<crate::graphics::ResolveInstance>,
}

impl World {
    pub fn new() -> Self {
        let player = Player::new(
            Vector2::new(0.0, 0.0)
        );
        let basic_enemy = BasicEnemy::new(
            Vector2::new(2.0, 0.0)
        );

        // let stage_left = Stage::new(
        //     Vector2::new(-2.0, 0.0),
        //     Vector2::new(0.25, 1.0),
        // );

        // let stage_right = Stage::new(
        //     Vector2::new(1.0, 0.0),
        //     Vector2::new(0.25, 1.0),
        // );

        // let stage_top = Stage::new(
        //     Vector2::new(0.0, 1.0),
        //     Vector2::new(1.0, 0.25),
        // );

        // let stage_down = Stage::new(
        //     Vector2::new(0.0, 5.0),
        //     Vector2::new(5.0, 0.25),
        // );
        // let stage = vec![stage_left, stage_right, stage_top, stage_down];

        // let objects: Vec<Box<dyn GameObject>> = vec![player, stage_left, stage_right, stage_top, stage_down];

        // let objects = objects.into_iter().map(|obj| (obj.get_uuid(), obj)).collect();
        let mut stage = HashMap::new();
        stage.insert(Uuid::new_v4(), Stage::new());
        Self {
            player,
            basic_enemy,
            stage,
            debug_objects: vec![],
        }
    }

    // don't we need a thing to tell it how much to change?
    pub fn update(&mut self, delta_time: f32, input_state: &crate::InputState, chatbox: &mut Chatbox) {
        // place blocks
        {
            use stage::TileType::*;
            let pos = input_state.mouse_position;
            let rounded = Vector2::new((pos.x).floor() as i32, (pos.y).floor() as i32);
            if input_state.mouse_pos_edge.contains(&MouseButton::Left) {
                self.stage.values_mut().next().map(|stage| stage.set_tile(&rounded, Some(Dirt)));
            }
            if input_state.mouse_pos_edge.contains(&MouseButton::Right) {
                self.stage.values_mut().next().map(|stage| stage.set_tile(&rounded, None));
            }
            self.debug_objects = vec![
                ResolveInstance {
                    overlaps: 0,
                    color: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    position: Vector2::new(rounded.x as f32, rounded.y as f32) + Vector2::new(0.5, 0.5),
                    scale: Vector2::new(1.0, 1.0),
                }
            ];
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
        self.player.update(delta_time, input_state);

        self.basic_enemy.update(delta_time);

        // update
        // for (id, object) in self.objects {
        //     
        // }

        self.physics(delta_time);
    }

    fn physics(&mut self, delta_time: f32) {
        // gather all who want to be physic'd
        let mut to_physics_on: Vec<&mut dyn Physics> = vec![
            &mut self.player,
            &mut self.basic_enemy,
        ];
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

