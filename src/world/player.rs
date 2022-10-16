use cgmath::{Vector2, Vector4};
use uuid::Uuid;
use winit::event::VirtualKeyCode;

use crate::{bounding_box::BoundingBox, InputState};

use super::{PhysicsObject, Projectile, GameObject, IDObject, Physics};

pub enum Direction {
    Left, Right
}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    Jumping(f32),
    Falling,
    OnGround
}

pub struct Player {
    id: Uuid,
    pub physics: PhysicsObject,
    pub state: State,
}

impl Player {
    const JUMP_SPEED: f32 = 4.0;
    const JUMP_HOLD_TIMER_MAX: f32 = 0.3;
    const JUMP_HOLD_TIMER_MIN: f32 = 0.15;

    const FALL_SPEED: f32 = 5.0;
    const PLAYER_ACCEL_Y: f32 = 22.0;
    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>) -> Self {
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, 1.0, 1.0),
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
        // change jump state
        self.state = match (input_state.key_pos_edge.contains(&VirtualKeyCode::Space),
                            input_state.key_down.contains(&VirtualKeyCode::Space),
                            self.state.clone()) {
            // case where we start jumping
            (true, _, State::OnGround) =>
                State::Jumping(0.0),

            // case where we keep jumping
            (_, false, State::Jumping(timer)) if timer < Player::JUMP_HOLD_TIMER_MIN =>
                State::Jumping(timer + delta_time),
            (_, true, State::Jumping(timer)) if timer < Player::JUMP_HOLD_TIMER_MAX =>
                State::Jumping(timer + delta_time),

            // go from jumping to falling
            (_, _, State::Jumping(_)) =>
                State::Falling,

            // jumping is not involved, leave it alone
            (_, _, state) => state,
        };

        // find target y velocity
        let target_vel_y = match self.state {
            State::Jumping(_) => -Player::JUMP_SPEED,
            _ => Player::FALL_SPEED,
        };

        // find acceleration in y
        let accel_y = if self.state == State::Jumping(0.0) {
            f32::INFINITY // this means velocity override
        } else {
            Player::PLAYER_ACCEL_Y
        } * delta_time;

        // move player to match target velocity y
        if f32::abs(self.physics.velocity.y - target_vel_y) < accel_y {
            self.physics.velocity.y = target_vel_y;
        } else {
            self.physics.velocity.y += f32::signum(target_vel_y - self.physics.velocity.y) * accel_y;
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

