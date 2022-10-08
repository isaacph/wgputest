use std::collections::HashSet;

use cgmath::{Zero, EuclideanSpace};
use winit::event::{WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub center: cgmath::Point2<f32>,
    pub zoom: f32,
    pub window_size: cgmath::Vector2<u32>,
}

impl Camera {
    pub fn new(window_size: cgmath::Vector2<u32>, zoom: f32) -> Self {
        Self {
            center: cgmath::Point2::new(0.0, 0.0),
            zoom,
            window_size,
        }
    }

    pub fn zoom_factor(&self) -> f32 {
        std::cmp::min(self.window_size.x, self.window_size.y) as f32 / f32::max(0.001, self.zoom)
    }

    pub fn camera_center_offset(&self) -> cgmath::Vector2<f32> {
        cgmath::Vector2::new(
            (self.window_size.x as f32) / 2.0,
            (self.window_size.y as f32) / 2.0
        )
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let zoom_factor = self.zoom_factor();
        let camera_center_offset = self.camera_center_offset();
        // let proj = cgmath::Matrix4::identity();
        // let view = cgmath::Matrix4::identity();
        let proj = cgmath::ortho(0.0, self.window_size.x as f32, self.window_size.y as f32, 0.0, 0.0, 1.0);
        let view = 
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(camera_center_offset.x, camera_center_offset.y, 0.0)) *
            cgmath::Matrix4::from_scale(zoom_factor) *
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(-self.center.x, -self.center.y, 0.0));
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn view_to_world_pos(&self, position: cgmath::Point2<f32>) -> cgmath::Point2<f32> {
        self.center + ((position - self.camera_center_offset()).to_vec() / self.zoom_factor())
    }
    
    pub fn view_to_world_scale(&self, scale: cgmath::Vector2<f32>) -> cgmath::Vector2<f32> {
        scale / self.zoom_factor()
    }

    pub fn world_to_view_pos(&self, position: cgmath::Point2<f32>) -> cgmath::Vector2<f32> {
        (position - self.center) * self.zoom_factor() + self.camera_center_offset()
    }
    
    pub fn world_to_view_scale(&self, scale: cgmath::Vector2<f32>) -> cgmath::Vector2<f32> {
        scale * self.zoom_factor()
    }
}

pub struct CameraController {
    speed: f32,
    inputs: HashSet<VirtualKeyCode>,
    relevant_inputs: HashSet<VirtualKeyCode>,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        use VirtualKeyCode::*;
        Self {
            speed,
            inputs: Default::default(),
            relevant_inputs: vec![W, A, S, D, Space, LShift].into_iter().collect(),
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
        use VirtualKeyCode::{W, A, S, D};
        use cgmath::Vector2;

        let mut dir = Vector2::zero();
        if self.inputs.contains(&W) { dir -= Vector2::unit_y(); }
        if self.inputs.contains(&S) { dir += Vector2::unit_y(); }
        if self.inputs.contains(&D) { dir += Vector2::unit_x(); }
        if self.inputs.contains(&A) { dir -= Vector2::unit_x(); }
        if dir != Vector2::zero() {
            dir = dir.normalize();
            let change = dir * self.speed * delta_time;
            camera.center += change;
        }
    }
}
