use cgmath::{Vector2, Vector4};
use uuid::Uuid;
use winit::event::VirtualKeyCode;

use crate::{bounding_box::BoundingBox, InputState};

use super::{PhysicsObject, Projectile, GameObject, IDObject, Physics};

pub enum Direction {
    Left, Right
}

#[derive(PartialEq, Debug)]
pub enum State {
    Jumping,
    Falling,
    OnGround
}

pub struct Player {
    id: Uuid,
    pub physics: PhysicsObject,
    pub state: State,
}

impl Player {
    const JUMP_SPEED: f32 = 2.0;
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>) -> Self {
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, scale.x, scale.y),
            velocity: Vector2::new(0.0, 0.0),
            can_move: true,
        };
        Self {
            id: Uuid::new_v4(),
            physics,
            state: State::Falling,
        }
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

    pub fn update(&mut self, delta_time: f32, input_state: &InputState) {
        // check for jump
        if input_state.key_pos_edge.contains(&VirtualKeyCode::Space) {
            // jump
            if self.state == State::OnGround {
                self.physics.velocity.y = -Player::JUMP_SPEED;
            }
        }

        // find player's ability to self-accelerate x
        let accel_x = delta_time * 10.0;

        // find target velocity x
        let mut target_vel_x = 0.0;
        if input_state.key_down.contains(&VirtualKeyCode::D) &&
            !input_state.key_down.contains(&VirtualKeyCode::A) {
            // strafe right
            target_vel_x += 1.0;
        }
        if input_state.key_down.contains(&VirtualKeyCode::A) &&
            !input_state.key_down.contains(&VirtualKeyCode::D) {
            // strafe left
            target_vel_x -= 1.0;
        }

        // move player to match target velocity x
        if f32::abs(self.physics.velocity.x - target_vel_x) < accel_x {
            self.physics.velocity.x = target_vel_x;
        } else {
            self.physics.velocity.x += f32::signum(target_vel_x - self.physics.velocity.x) * accel_x;
        }

        let gravity = 5.0; // placeholder
        self.physics.velocity += Vector2::unit_y() * delta_time * gravity;
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

    fn pre_physics(&mut self) {
        if self.state == State::OnGround {
            self.state = State::Falling;
        }
    }

    fn resolve(&mut self, delta: Vector2<f32>, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(delta + resolve);
        if resolve.y < 0.0 {
            // on colliding with the ground
            self.physics.velocity.y = f32::min(self.physics.velocity.y, 0.0);
            self.state = State::OnGround;
        }
        if resolve.y > 0.0 {
            // on colliding with the ceiling
            self.physics.velocity.y = f32::max(self.physics.velocity.y, 0.0);
        }
        delta + resolve
    }
}

