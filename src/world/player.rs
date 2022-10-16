use std::collections::HashSet;

use cgmath::{Vector2, Vector4};
use uuid::Uuid;
use winit::event::VirtualKeyCode;

use crate::{bounding_box::BoundingBox, InputState};

use super::{PhysicsObject, GameObject, IDObject, Physics, physics::PhysObjType, projectile::{Projectile, ProjectileType}};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right
}

impl Direction {
    pub fn value(&self) -> f32 {
        match *self {
            Direction::Left => -1.0,
            Direction::Right => 1.0
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AerialState {
    Jumping(f32),
    Falling,
    OnGround
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum HorizontalState {
    MovingLeft,
    MovingRight,
    TurningLeft,
    TurningRight,
    Stopping,
    Stopped
}

pub struct DashInfo {
    pub num_dashes_left: u32,
    pub dashing_time_elapsed: f32,
    pub cooldown_time_elapsed: f32,
    pub velocity: Vector2<f32>,
}

impl DashInfo {
    const NUM_DASHES: u32 = 1;

    const DASH_SPEED: f32 = 20.0;
    const DASH_ACCEL: f32 = 50.0;
    const DASH_SLOWDOWN_MULTIPLIER: f32 = 10.0;
    const DASH_DURATION: f32 = 0.2;
    // DASH_PEAK begins dash slowdown 
    const DASH_PEAK: f32 = 0.1;
    // DASH_HANG_TIME comes after dash peak but before end of dash_duration 
    const DASH_HANG_TIME: f32 = 0.05;
    const DASH_COOLDOWN: f32 = 0.1;

    pub fn new() -> Self {
        Self {   
            num_dashes_left: 1,
            dashing_time_elapsed: 0.0,
            cooldown_time_elapsed: 0.0,
            velocity: Vector2::new(0.0, 0.0) 
        }
    } 

    pub fn has_dashes_remaining(&self) -> bool {
        self.num_dashes_left > 0
    }

    pub fn is_dashing(&self) -> bool {
        self.dashing_time_elapsed > 0.0 && !self.is_in_cooldown()
    }

    pub fn is_before_peak(&self) -> bool {
        self.dashing_time_elapsed >= 0.0 && self.dashing_time_elapsed < DashInfo::DASH_PEAK
    }

    pub fn is_after_peak(&self) -> bool {
        self.is_dashing() && !self.is_before_peak()
    }

    pub fn is_hanging(&self) -> bool {
        self.is_dashing() && self.dashing_time_elapsed > DashInfo::DASH_DURATION - DashInfo::DASH_HANG_TIME 
    }

    pub fn is_dashing_vertically(&self) -> bool {
        self.is_dashing() && self.velocity.y != 0.0
    }

    pub fn is_dashing_horizontally(&self) -> bool {
        self.is_dashing() && self.velocity.x != 0.0
    }

    pub fn is_in_cooldown(&self) -> bool {
        self.cooldown_time_elapsed > 0.0
    }

    pub fn update_cooldown(&mut self, delta_time: f32) {
        self.cooldown_time_elapsed += delta_time;
        if self.cooldown_time_elapsed >= DashInfo::DASH_COOLDOWN {
            self.cooldown_time_elapsed = 0.0;
        }
    }
}

pub struct Player {
    id: Uuid,
    dash_info: DashInfo,
    pub physics: PhysicsObject,
    pub direction: Direction,
    pub aerial_state: AerialState,
    pub horizontal_state: HorizontalState,
    pub alive: bool,
    pub current_projectile: usize,
}

impl Player {
    const JUMP_SPEED: f32 = 4.0;
    const JUMP_HOLD_TIMER_MAX: f32 = 0.3;
    const JUMP_HOLD_TIMER_MIN: f32 = 0.15;

    const FALL_SPEED: f32 = 5.0;
    const PLAYER_ACCEL_Y: f32 = 22.0;

    const PLAYER_MOVE_SPEED_X: f32 = 7.0;
    const PLAYER_ACCEL_X: f32 = 10.0;
    const PLAYER_ON_GROUND_MULTIPLIER_X: f32 = 2.0;
    const PLAYER_TURNAROUND_MULTIPLIER_X: f32 = 14.0; 

    // initialize with position, scale, and color -- velocity and acceleration should be 0 when starting
    pub fn new(position: Vector2<f32>) -> Self {
        let physics = PhysicsObject {
            bounding_box: BoundingBox::new(position, 1.0, 1.0),
            velocity: Vector2::new(0.0, 0.0),
            can_move: true,
            typ: PhysObjType::Player,
            collides_with: PhysObjType::all(),
            move_by: vec![PhysObjType::Wall].into_iter().collect(),
        };
        Self {
            id: Uuid::new_v4(),
            dash_info: DashInfo::new(),
            physics,
            direction: Direction::Right,
            aerial_state: AerialState::Falling,
            horizontal_state: HorizontalState::Stopped,
            alive: true,
            current_projectile: 1,
        }
    }

    pub fn update_dash(&mut self, delta_time: f32, input_state: &InputState) {
        let accel_scalar = if self.dash_info.dashing_time_elapsed == 0.0 {
            f32::INFINITY
            // DashInfo::DASH_ACCEL
        } else if self.dash_info.is_after_peak() {
            DashInfo::DASH_ACCEL * DashInfo::DASH_SLOWDOWN_MULTIPLIER
        // } 
        // else if self.dash_info.is_hanging() {
        //     0.0
        } else {
            DashInfo::DASH_ACCEL
        } * delta_time;

        let accel = match (input_state.key_pos_edge.contains(&VirtualKeyCode::W), 
                        input_state.key_pos_edge.contains(&VirtualKeyCode::A),
                        input_state.key_pos_edge.contains(&VirtualKeyCode::S),
                        input_state.key_pos_edge.contains(&VirtualKeyCode::D), 
                        input_state.key_down.contains(&VirtualKeyCode::W), 
                        input_state.key_down.contains(&VirtualKeyCode::A),
                        input_state.key_down.contains(&VirtualKeyCode::S),
                        input_state.key_down.contains(&VirtualKeyCode::D)) {
            // left or right
            (false, true, false, _, _, _, _, _) 
            | (_, _, _, _, false, true, false, _) 
            | (false, _, false, true, _, _, _, _) 
            | (_, _, _, _, false, _, false, true) => Vector2::new(accel_scalar, 0.0),

            // up or down
            (_, false, true, false, _, _, _, _)
            | (_, _, _, _, _, false, true, false) 
            | (true, false, _, false, _, _, _, _) 
            | (_, _, _, _, true, false, _, false) => Vector2::new(0.0, accel_scalar),

            // diagonals are sqrt 2 to feel good? (circular)
            (false, true, true, false, _, _, _, _)
            | (_, _, _, _, false, true, true, false)
            | (false, false, true, true, _, _, _, _)
            | (_, _, _, _, false, false, true, true)
            | (true, true, false, false, _, _, _, _) 
            | (_, _, _, _, true, true, false, false)
            | (true, false, false, true, _, _, _, _)
            | (_, _, _, _, true, false, false, true)  => Vector2::new(accel_scalar / f32::sqrt(2.0), accel_scalar / f32::sqrt(2.0)),

            (_, _, _, _, _, _, _, _) => Vector2::new(accel_scalar, 0.0),
        };

        let target_speed = if self.dash_info.is_before_peak() {
            DashInfo::DASH_SPEED
        } else {
            0.0
        };

        let target_vel = match (input_state.key_pos_edge.contains(&VirtualKeyCode::W), 
                        input_state.key_pos_edge.contains(&VirtualKeyCode::A),
                        input_state.key_pos_edge.contains(&VirtualKeyCode::S),
                        input_state.key_pos_edge.contains(&VirtualKeyCode::D), 
                        input_state.key_down.contains(&VirtualKeyCode::W), 
                        input_state.key_down.contains(&VirtualKeyCode::A),
                        input_state.key_down.contains(&VirtualKeyCode::S),
                        input_state.key_down.contains(&VirtualKeyCode::D)) {
            // left
            (false, true, false, _, _, _, _, _)
            | (_, _, _, _, false, true, false, _) => Vector2::new(-target_speed, 0.0),

            // right
            (false, _, false, true, _, _, _, _)
            | (_, _, _, _, false, _, false, true) => Vector2::new(target_speed, 0.0),

            // down
            (_, false, true, false, _, _, _, _)
            | (_, _, _, _, _, false, true, false) => Vector2::new(0.0, target_speed),
            
            // up
            (true, false, _, false, _, _, _, _) 
            | (_, _, _, _, true, false, _, false) => Vector2::new(0.0, -target_speed),

            // down-left
            (false, true, true, false, _, _, _, _)
            | (_, _, _, _, false, true, true, false) => Vector2::new(-target_speed / f32::sqrt(2.0), target_speed / f32::sqrt(2.0)),
            
            // down-right
            (false, false, true, true, _, _, _, _)
            | (_, _, _, _, false, false, true, true) => Vector2::new(target_speed / f32::sqrt(2.0), target_speed / f32::sqrt(2.0)),
            
            // up-left
            (true, true, false, false, _, _, _, _) 
            | (_, _, _, _, true, true, false, false) => Vector2::new(-target_speed / f32::sqrt(2.0), -target_speed / f32::sqrt(2.0)),
            
            // up-right
            (true, false, false, true, _, _, _, _)
            | (_, _, _, _, true, false, false, true) => Vector2::new(target_speed / f32::sqrt(2.0), -target_speed / f32::sqrt(2.0)),
            
            // diagonals are sqrt 2 to feel good? (circular)

            // default dash should be whichever way the character is facing
            (_, _, _, _, _, _, _, _) => Vector2::new(target_speed * self.direction.value(), 0.0),
        };

        println!("dashing with velocity ({}, {}) at time {}", target_vel.x, target_vel.y, self.dash_info.dashing_time_elapsed);
        
        // "reset" velocities in the relevant directions if we just started dashing
        // for diagonal, these should be halved or set to some low value instead of completely zeroing out
        self.physics.velocity.y = if target_vel.y == 0.0 {
            0.0
        } else { 
            self.physics.velocity.y 
        };
        self.physics.velocity.x = if target_vel.x == 0.0 {
            0.0
        } else { 
            self.physics.velocity.x 
        };
        

        if f32::abs(self.physics.velocity.x - target_vel.x) < accel.x {
            self.physics.velocity.x = target_vel.x;
        } else {
            self.physics.velocity.x += f32::signum(target_vel.x - self.physics.velocity.x) * accel.x;
        }
        if f32::abs(self.physics.velocity.y - target_vel.y) < accel.y {
            self.physics.velocity.y = target_vel.y;
        } else {
            self.physics.velocity.y += f32::signum(target_vel.y - self.physics.velocity.y) * accel.y;
        }

        // mimic the code directly above but for the y directions
        // if f32::abs(self.physics.velocity.x - target_vel.x) < accel.x {
        //     self.physics.velocity.x = target_vel.x;
        // } else {
        //     self.physics.velocity.x += f32::signum(target_vel.x - self.physics.velocity.x) * accel.x;
        // }
        
        self.dash_info.dashing_time_elapsed += delta_time;
        if self.dash_info.dashing_time_elapsed >= DashInfo::DASH_DURATION + DashInfo::DASH_HANG_TIME {
            self.dash_info.velocity.x = 0.0; 
            self.dash_info.velocity.y = 0.0;
            self.dash_info.dashing_time_elapsed = 0.0;

            self.dash_info.cooldown_time_elapsed += delta_time;
        }
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
        // update projectile type
        self.current_projectile = match (input_state.key_pos_edge.contains(&VirtualKeyCode::Key1),
                                        input_state.key_pos_edge.contains(&VirtualKeyCode::Key2),
                                        input_state.key_down.contains(&VirtualKeyCode::Key1),
                                        input_state.key_down.contains(&VirtualKeyCode::Key2)) {
            (true, _, _, _)
            | (_, _, true, _) => 0,

            (_, true, _, _)
            | (_, _, _, true) => 0,

            (_, _, _, _) => self.current_projectile
        };


        self.direction = match (input_state.key_pos_edge.contains(&VirtualKeyCode::A),
                                input_state.key_pos_edge.contains(&VirtualKeyCode::D),
                                input_state.key_down.contains(&VirtualKeyCode::A),
                                input_state.key_down.contains(&VirtualKeyCode::D)) {
            (true, _, _, _) 
            | (_, _, true, _) => Direction::Left,
            
            (_, true, _, _) 
            | (_, _, _, true) => Direction::Right,

            (_,_,_,_) => self.direction
        };
        

        // change jump state
        self.aerial_state = match (input_state.key_pos_edge.contains(&VirtualKeyCode::Space),
                            input_state.key_down.contains(&VirtualKeyCode::Space),
                            self.aerial_state.clone()) {
            // case where we start jumping
            (true, _, AerialState::OnGround) =>
                AerialState::Jumping(0.0),

            // case where we keep jumping
            (_, false, AerialState::Jumping(timer)) if timer < Player::JUMP_HOLD_TIMER_MIN =>
                AerialState::Jumping(timer + delta_time),
            (_, true, AerialState::Jumping(timer)) if timer < Player::JUMP_HOLD_TIMER_MAX =>
                AerialState::Jumping(timer + delta_time),

            // go from jumping to falling
            (_, _, AerialState::Jumping(_)) =>
                AerialState::Falling,

            // jumping is not involved, leave it alone
            (_, _, state) => state,
        };

        self.horizontal_state = match (input_state.key_pos_edge.contains(&VirtualKeyCode::A),
                                input_state.key_pos_edge.contains(&VirtualKeyCode::D),
                                input_state.key_down.contains(&VirtualKeyCode::A),
                                input_state.key_down.contains(&VirtualKeyCode::D),
                                self.horizontal_state.clone() 
        ) {
            //  BUG: this still lets the player vibrate back and forth slowly towards one direction
            //       reproduce by pressing both A, D together
            (_, _, true, true, state) if state != HorizontalState::Stopped => HorizontalState::Stopping, 
            // inputting A while moving right initiates turning
            (true, false, _, _, HorizontalState::MovingRight)
            | (_, _, true, false, HorizontalState::MovingRight) => HorizontalState::TurningLeft,
            // inputting D while moving left initiates turning
            (false, true, _, _, HorizontalState::MovingLeft)
            | (_, _, false, true, HorizontalState::MovingLeft) => HorizontalState::TurningRight,

            (true, false, _, _, HorizontalState::MovingLeft) 
            | (true, false, _, _, HorizontalState::TurningRight) 
            | (true, false, _, _, HorizontalState::Stopping) 
            | (true, false, _, _, HorizontalState::Stopped)
            | (_, _, true, false, HorizontalState::MovingLeft)
            | (_, _, true, false, HorizontalState::TurningRight)
            | (_, _, true, false, HorizontalState::Stopping)
            | (_, _, true, false, HorizontalState::Stopped) => HorizontalState::MovingLeft,

            (false, true, _, _, HorizontalState::MovingRight) 
            | (false, true, _, _, HorizontalState::TurningLeft) 
            | (false, true, _, _, HorizontalState::Stopping) 
            | (false, true, _, _, HorizontalState::Stopped)
            | (_, _, false, true, HorizontalState::MovingRight) 
            | (_, _, false, true, HorizontalState::TurningLeft) 
            | (_, _, false, true, HorizontalState::Stopping)
            | (_, _, false, true, HorizontalState::Stopped) => HorizontalState::MovingRight,


            // no inputs is stopping unless stoped
            (false, false, false, false, state) if state != HorizontalState::Stopped => HorizontalState::Stopping,

            (_, _, _, _, state) => state,

        };

        // find target y velocity
        let target_vel_y = match self.aerial_state {
            AerialState::Jumping(_) => -Player::JUMP_SPEED,
            _ => Player::FALL_SPEED,
        };

        // find acceleration in y
        let accel_y = if self.aerial_state == AerialState::Jumping(0.0) {
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
        // when aerial state is OnGround, physics should feel snappier -- higher acceleration
        let accel_x = if self.aerial_state == AerialState::OnGround 
         && (self.horizontal_state == HorizontalState::TurningLeft 
            || self.horizontal_state == HorizontalState::TurningRight 
            || self.horizontal_state == HorizontalState::Stopping){
            delta_time * Player::PLAYER_ACCEL_X * Player::PLAYER_ON_GROUND_MULTIPLIER_X   
        } else {
            delta_time * Player::PLAYER_ACCEL_X
        };

        let target_vel_x = match self.horizontal_state {
            HorizontalState::MovingLeft => -Player::PLAYER_MOVE_SPEED_X,
            HorizontalState::TurningLeft => -Player::PLAYER_MOVE_SPEED_X * Player::PLAYER_TURNAROUND_MULTIPLIER_X,
            HorizontalState::MovingRight => Player::PLAYER_MOVE_SPEED_X,
            HorizontalState::TurningRight => Player::PLAYER_MOVE_SPEED_X * Player::PLAYER_TURNAROUND_MULTIPLIER_X,
            _ => 0.0
        };

        // move player to match target velocity x
        if f32::abs(self.physics.velocity.x - target_vel_x) < accel_x {
            self.physics.velocity.x = target_vel_x;
        } else {
            self.physics.velocity.x += f32::signum(target_vel_x - self.physics.velocity.x) * accel_x;
        }

        // update horizontal state depending on current velocity sign, target velocity sign, and state
        self.horizontal_state = match (self.physics.velocity.x, target_vel_x) {
            (current, target) if current > 0.0 && target < 0.0 
            => HorizontalState::TurningLeft,

            (current, target) if current > 0.0 && target > 0.0 
            => HorizontalState::MovingRight,

            (current, target) if current < 0.0 && target > 0.0 
            => HorizontalState::TurningRight,

            (current, target) if current < 0.0 && target < 0.0
                => HorizontalState::MovingLeft,
            
            (current, target) if current != 0.0 && target == 0.0
                => HorizontalState::Stopping,

            (current, target) if current == 0.0 && target == 0.0
                => HorizontalState::Stopped,

            (_, _) => self.horizontal_state
        };


        // handle dash stuff here
        // is this the best place to handle dash?

        if input_state.key_pos_edge.contains(&VirtualKeyCode::E) 
            && !(self.dash_info.is_dashing() || self.dash_info.is_in_cooldown())
            && self.dash_info.has_dashes_remaining() {
            self.dash_info.num_dashes_left -= 1;
            self.update_dash(delta_time, input_state);
        } else if self.dash_info.is_dashing() {
            self.update_dash(delta_time, input_state);
        } else if self.dash_info.is_in_cooldown() {
            self.dash_info.update_cooldown(delta_time);
        } else if self.aerial_state == AerialState::OnGround {
            self.dash_info.num_dashes_left = DashInfo::NUM_DASHES;
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
    fn get_physics(&self) -> Vec<(Uuid, PhysicsObject)> {
        vec![(self.id, self.physics.clone())]
    }

    fn pre_physics(&mut self) {
        if self.aerial_state == AerialState::OnGround {
            self.aerial_state = AerialState::Falling;
        }
    }

    fn resolve(&mut self, _: Uuid, delta: Vector2<f32>, resolve: Vector2<f32>, types: Vec<(PhysObjType, Uuid)>) -> Vector2<f32> {
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
            self.physics.velocity.x = 0.0;
            self.horizontal_state = HorizontalState::Stopped;
        }
        delta + resolve
    }

    fn typ(&self) -> PhysObjType {
        PhysObjType::Player
    }
}

