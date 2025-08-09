use std::sync::Arc;

use wgpu::WasmNotSend;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
    window::Window,
};

pub trait WgpuAppAction {
    fn new(window: Arc<Window>) -> impl core::future::Future<Output = Self> + WasmNotSend;

    fn set_window_size(&mut self, new_size: PhysicalSize<u32>);

    fn get_size(&self) -> PhysicalSize<u32>;

    fn keyboard_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn mouse_click(&mut self, _state: ElementState, _button: MouseButton) -> bool {
        false
    }

    fn mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) -> bool {
        false
    }

    fn cursor_move(&mut self, _position: PhysicalPosition<f64>) -> bool {
        false
    }

    fn device_input(&mut self, _event: &DeviceEvent) -> bool {
        false
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}
