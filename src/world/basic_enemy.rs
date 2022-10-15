use cgmath::{Vector2, Vector4};
use uuid::Uuid;
use crate::{graphics::textured::Instance, bounding_box::BoundingBox};
use super::{GameObjectData, GameObject, IDObject, Physics};

pub struct BasicEnemy {
    id: Uuid,
    data: GameObjectData
}

impl BasicEnemy {
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
        Self {
            data: GameObjectData::new(position, scale, color),
            id: Uuid::new_v4()
        }
    }
}

impl GameObject for BasicEnemy {
    fn get_instance(&self) -> Instance {
        return self.data.get_instance();
    }
}

impl IDObject for BasicEnemy {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for BasicEnemy {
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
        self.data.bounding_box.center += delta_position;
    }

    fn can_move(&self) -> bool {
        return true;
    }
}
