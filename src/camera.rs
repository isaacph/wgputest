use std::collections::HashSet;

use cgmath::{Vector3, EuclideanSpace, Vector4, Zero};
use winit::event::{WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub pos: cgmath::Point3<f32>,
    pub rot: cgmath::Vector3<cgmath::Rad<f32>>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            pos: cgmath::Point3::new(0.0, 0.0, -1.0),
            rot: cgmath::Vector3::new(cgmath::Rad(0.0), cgmath::Rad(0.0), cgmath::Rad(0.0)),
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        let view =
            cgmath::Matrix4::from_axis_angle(Vector3::unit_x(), self.rot.x) *
            cgmath::Matrix4::from_axis_angle(Vector3::unit_y(), self.rot.y) *
            cgmath::Matrix4::from_axis_angle(Vector3::unit_z(), self.rot.z) *
            cgmath::Matrix4::from_translation(self.pos.to_vec());

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn forward(&self) -> cgmath::Vector3<f32> {
        use cgmath::InnerSpace;
        let rot = 
            cgmath::Matrix4::from_axis_angle(Vector3::unit_x(), -self.rot.x) *
            cgmath::Matrix4::from_axis_angle(Vector3::unit_y(), -self.rot.y) *
            cgmath::Matrix4::from_axis_angle(Vector3::unit_z(), -self.rot.z);
        let v4 = rot * Vector4::unit_z();
        return v4.truncate().normalize();
    }
}

pub struct CameraController {
    speed: f32,
    rot_speed: f32,
    inputs: HashSet<VirtualKeyCode>,
    relevant_inputs: HashSet<VirtualKeyCode>,
}

impl CameraController {
    pub fn new(speed: f32, rot_speed: f32) -> Self {
        use VirtualKeyCode::*;
        Self {
            speed,
            rot_speed,
            inputs: Default::default(),
            relevant_inputs: vec![W, A, S, D, Q, E, F, R, Space, LShift].into_iter().collect(),
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(key), ..
                }, ..
            } if self.relevant_inputs.contains(&key) => {
                match state {
                    ElementState::Pressed => self.inputs.insert(*key),
                    ElementState::Released => self.inputs.remove(key),
                };
                true
            },
            _ => false,
        }
    }

    pub fn update_camera(&self, delta_time: f32, camera: &mut Camera) {
        use cgmath::InnerSpace;
        use VirtualKeyCode::{W, A, S, D, Q, E, F, R, Space, LShift};

        let y_rot = {
            let mut x = 0.0;
            if self.inputs.contains(&E) {
                x += 1.0;
            }
            if self.inputs.contains(&Q) {
                x -= 1.0;
            }
            x
        };
        let x_rot = {
            let mut x = 0.0;
            if self.inputs.contains(&F) {
                x += 1.0;
            }
            if self.inputs.contains(&R) {
                x -= 1.0;
            }
            x
        };
        camera.rot.y += cgmath::Rad(y_rot * self.rot_speed * delta_time);
        camera.rot.x += cgmath::Rad(x_rot * self.rot_speed * delta_time);

        let forward = {
            let mut f = camera.forward();
            f.y = 0.0;
            f.normalize()
        };
        println!("Forward: {}, {}, {}", forward.x, forward.y, forward.z);
        let right = forward.cross(Vector3::unit_y());
        // println!("Right: {}, {}, {}", right.x, right.y, right.z);

        let mut dir = Vector3::zero();
        if self.inputs.contains(&W) { dir += forward; }
        if self.inputs.contains(&S) { dir -= forward; }
        if self.inputs.contains(&D) { dir += right; }
        if self.inputs.contains(&A) { dir -= right; }
        if dir != Vector3::zero() {
            dir = dir.normalize();
            let change = dir * self.speed * delta_time;
            println!("Dir: {}, {}, {}", dir.x, dir.y, dir.z);
            println!("Change: {}, {}, {}", change.x, change.y, change.z);

            camera.pos += change;
        }

        let up_down = {
            let mut x = 0.0;
            if self.inputs.contains(&Space) {
                x -= 1.0;
            }
            if self.inputs.contains(&LShift) {
                x += 1.0;
            }
            x
        };
        camera.pos.y += up_down * self.speed * delta_time;
    }
}
