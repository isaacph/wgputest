use cgmath::Vector2;
use uuid::Uuid;

use crate::bounding_box::BoundingBox;

use super::{physics::{PhysicsObject, Physics, PhysObjType}, GameObject, IDObject};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectileType {
    Basic
}
impl ProjectileType {
    pub fn all() -> Vec<ProjectileType> {
        use ProjectileType::*;
        vec![Basic]
    }
}

pub struct Projectile {
    id: Uuid,
    physics: PhysicsObject,
    pub alive: bool,
    typ: ProjectileType,
    collides_with: PhysObjType,
}


impl Projectile {
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, velocity: Vector2<f32>, collides_with: PhysObjType) -> Self {
        let typ = ProjectileType::Basic;
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, scale.x, scale.y),
            velocity,
            can_move: true,
            typ: super::physics::PhysObjType::Projectile(typ),
            collides_with: vec![collides_with, PhysObjType::Wall].into_iter().collect(),
        };
        Self {
            id: Uuid::new_v4(),
            physics,
            alive: true,
            typ,
            collides_with,
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
    fn get_physics(&self) -> Vec<(Uuid, PhysicsObject)> {
        vec![(self.id, self.physics.clone())]
    }

    fn resolve(&mut self, _: Uuid, delta: Vector2<f32>, resolve: Vector2<f32>, types: Vec<PhysObjType>) -> Vector2<f32> {
        self.physics.bounding_box.add(delta + resolve);
        delta + resolve
    }

    fn pre_physics(&mut self) {
    }

    fn typ(&self) -> PhysObjType {
        PhysObjType::Projectile(ProjectileType::Basic)
    }
}

