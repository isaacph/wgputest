use cgmath::{Vector2, Vector4};
use uuid::Uuid;
use winit::event::VirtualKeyCode;
use crate::{bounding_box::BoundingBox, InputState};
use super::{GameObject, IDObject, Physics, physics::PhysicsObject};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AerialState {
    Jumping(f32),
    Falling,
    OnGround
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Direction {
    Left, Right
}

impl Direction {
    pub fn to_f32(&self) -> f32 {
        match *self {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
        }
    }
    pub fn from(x_dir: f32) -> Option<Direction> {
        if x_dir < 0.0 {
            Some(Direction::Left)
        } else if x_dir > 0.0 {
            Some(Direction::Right)
        } else {
            None
        }
    }
    pub fn reverse(&self) -> Self {
        match *self {
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }
}

pub struct BasicEnemy {
    id: Uuid,
    pub physics: PhysicsObject,
    pub aerial_state: AerialState,
    pub direction: Direction,
}

impl BasicEnemy {
    const JUMP_SPEED: f32 = 4.0;
    const JUMP_HOLD_TIMER_MAX: f32 = 0.3;
    const JUMP_HOLD_TIMER_MIN: f32 = 0.15;

    const FALL_SPEED: f32 = 5.0;
    const ACCEL_Y: f32 = 22.0;

    const ACCEL_X: f32 = 22.0;
    const MOVE_SPEED_X: f32 = 1.0;
    const VELOCITY_ON_GROUND_MULTIPLIER_X: f32 = 2.0;
    const ACCEL_ON_GROUND_MULTIPLIER_X: f32 = 2.0;

    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>) -> Self {
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, 0.9, 0.9),
            velocity: Vector2::new(0.0, 0.0),
            can_move: true,
        };
        Self {
            id: Uuid::new_v4(),
            physics,
            aerial_state: AerialState::Falling,
            direction: Direction::Left,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let jumping = false;
        let hold_jump = false;

        // change jump state
        self.aerial_state = match (jumping,
                            hold_jump,
                            self.aerial_state.clone()) {
            // case where we start jumping
            (true, _, AerialState::OnGround) =>
                AerialState::Jumping(0.0),

            // case where we keep jumping
            (_, false, AerialState::Jumping(timer)) if timer < Self::JUMP_HOLD_TIMER_MIN =>
                AerialState::Jumping(timer + delta_time),
            (_, true, AerialState::Jumping(timer)) if timer < Self::JUMP_HOLD_TIMER_MAX =>
                AerialState::Jumping(timer + delta_time),

            // go from jumping to falling
            (_, _, AerialState::Jumping(_)) =>
                AerialState::Falling,

            // jumping is not involved, leave it alone
            (_, _, state) => state,
        };

        // find target y velocity
        let target_vel_y = match self.aerial_state {
            AerialState::Jumping(_) => -Self::JUMP_SPEED,
            _ => Self::FALL_SPEED,
        };

        // find acceleration in y
        let accel_y = if self.aerial_state == AerialState::Jumping(0.0) {
            f32::INFINITY // this means velocity override
        } else {
            Self::ACCEL_Y
        } * delta_time;

        // move player to match target velocity y
        if f32::abs(self.physics.velocity.y - target_vel_y) < accel_y {
            self.physics.velocity.y = target_vel_y;
        } else {
            self.physics.velocity.y += f32::signum(target_vel_y - self.physics.velocity.y) * accel_y;
        }

        // find target x velocity
        let target_vel_x = match self.aerial_state {
            AerialState::OnGround =>
                Self::VELOCITY_ON_GROUND_MULTIPLIER_X,
            AerialState::Falling | AerialState::Jumping(_) =>
                1.0,
        } * self.direction.to_f32() * Self::MOVE_SPEED_X;

        // find acceleration in x
        let accel_x = match self.aerial_state {
            AerialState::OnGround =>
                Self::ACCEL_ON_GROUND_MULTIPLIER_X,
            AerialState::Falling | AerialState::Jumping(_) =>
                1.0,
        } * Self::ACCEL_X * delta_time;

        // move player to match target velocity x
        if f32::abs(self.physics.velocity.x - target_vel_x) < accel_x {
            self.physics.velocity.x = target_vel_x;
        } else {
            self.physics.velocity.x += f32::signum(target_vel_x - self.physics.velocity.x) * accel_x;
        }
    }
}

impl GameObject for BasicEnemy {
}

impl IDObject for BasicEnemy {
    fn get_uuid(&self) -> Uuid {
        return self.id;
    }
}

impl Physics for BasicEnemy {
    fn get_physics(&self) -> Vec<(Uuid, PhysicsObject)> {
        vec![(self.id, self.physics.clone())]
    }

    fn pre_physics(&mut self) {
        if self.aerial_state == AerialState::OnGround {
            self.aerial_state = AerialState::Falling;
        }
    }

    fn resolve(&mut self, _: Uuid, delta: Vector2<f32>, resolve: Vector2<f32>) -> Vector2<f32> {
        self.physics.bounding_box.add(delta + resolve);
        if resolve.y < 0.0 {
            // on colliding with the ground
            self.physics.velocity.y = f32::min(self.physics.velocity.y, 0.0);
            self.aerial_state = AerialState::OnGround;
        }
        if resolve.y > 0.0 {
            // on colliding with the ceiling
            self.physics.velocity.y = f32::max(self.physics.velocity.y, 0.0);
        }
        if resolve.x != 0.0 {
            // horizontal collision
            self.physics.velocity.x *= -1.0;
            self.direction = self.direction.reverse();
        }
        delta + resolve
    }
}

