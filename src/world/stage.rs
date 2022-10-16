use cgmath::{Vector2, Zero};
use uuid::Uuid;
use crate::bounding_box::BoundingBox;
use super::{physics::{PhysicsObject, Physics, PhysObjType}, GameObject, IDObject};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TileType {
    Dirt
}

pub struct Stage {
    id: Uuid,
    pub tiles: HashMap<Vector2<i32>, (Uuid, TileType)>,
}

impl Stage {
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            tiles: HashMap::new(),
        }
    }

    pub fn set_tile(&mut self, pos: &Vector2<i32>, value: Option<TileType>) {
        match value {
            None => self.tiles.remove(&pos),
            Some(typ) => self.tiles.insert(pos.clone(), (Uuid::new_v4(), typ)),
        };
        println!("new tiles len: {}", self.tiles.len());
    }

    pub fn get_tile(&self, pos: &Vector2<i32>) -> Option<TileType> {
        self.tiles.get(pos).map(|(_, typ)| *typ)
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
    fn get_physics(&self) -> Vec<(Uuid, PhysicsObject)> {
        self.tiles.clone().into_iter().map(|(tile, (id, _))|
            (id, PhysicsObject {
                bounding_box: BoundingBox::new(Vector2::new(tile.x as f32 + 0.5, tile.y as f32 + 0.5), 1.0, 1.0),
                can_move: false,
                velocity: Vector2::new(0.0, 0.0),
                typ: PhysObjType::Wall,
                collides_with: PhysObjType::all(),
                move_by: vec![PhysObjType::Wall].into_iter().collect(),
            })
        ).collect()
    }

    fn pre_physics(&mut self) {
    }

    fn resolve(&mut self, _: Uuid, delta: Vector2<f32>, resolve: Vector2<f32>, _: Vec<PhysObjType>) -> Vector2<f32> {
        cgmath::Vector2::zero()
    }

    fn typ(&self) -> super::physics::PhysObjType {
        todo!()
    }
}
