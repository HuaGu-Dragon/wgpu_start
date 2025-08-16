use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, NamedKey},
};

use crate::camera::Camera;

#[derive(Debug, Clone, Copy)]
pub struct PlayerController {
    pub speed: f32,
    pub forward: bool,
    pub left: bool,
    pub right: bool,
    pub backward: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            speed: 0.05,
            forward: false,
            left: false,
            right: false,
            backward: false,
        }
    }
}

impl PlayerController {
    pub fn handle_keyboard_input(&mut self, event: KeyEvent) {
        match event {
            KeyEvent {
                physical_key,
                logical_key,
                state,
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                match logical_key {
                    winit::keyboard::Key::Named(NamedKey::Space) => todo!(),
                    _ => {}
                }
                match physical_key {
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyW) => {
                        self.forward = pressed;
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyA) => {
                        self.left = pressed;
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyS) => {
                        self.backward = pressed;
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyD) => {
                        self.right = pressed;
                    }
                    _ => {}
                }
            }
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let camera_norm = forward.normalize();
        let forward_mag = forward.length();

        if self.forward && forward_mag > self.speed {
            camera.eye += camera_norm * self.speed;
        }
        if self.backward {
            camera.eye -= camera_norm * self.speed;
        }
    }
}
