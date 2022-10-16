use cgmath::{Vector2, Vector4};
use uuid::Uuid;

use crate::bounding_box::BoundingBox;

use super::{PhysicsObject, Projectile, GameObject, IDObject, Physics};

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

    pub fn jump(&mut self) {
        const JUMP_VELOCITY: Vector2<f32> = Vector2::new(0.0, 2.0);
        self.physics.velocity += JUMP_VELOCITY;
    }

    pub fn dash(&mut self) {
        const DASH_VELOCITY: Vector2<f32> = Vector2::new(2.0, 0.0);
        self.physics.velocity += DASH_VELOCITY;
    }

    // creates and returns a projectile to add to the game world
    pub fn shoot(&self) -> () {
        // // Wand is in front of the player
        // let WAND_LOCATION: Vector2<f32> = self.physics.bounding_box.center + self.direction;
        // return Projectile::new(
        //     WAND_LOCATION,
        //     Vector2::new(0.25, 0.25),
        //     Vector4::new(0.0, 0.0, 0.0, 0.0),
        //     self.direction
        // )
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

