use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::WasmNotSend;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::EventLoop,
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

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

struct WgpuAppHandler<A> {
    window: Option<Arc<Window>>,
    title: &'static str,
    app: Arc<Mutex<Option<A>>>,
    #[allow(dead_code)]
    missed_resize: Arc<Mutex<Option<PhysicalSize<u32>>>>,
}

impl<A> WgpuAppHandler<A> {
    fn new(title: &'static str) -> Self {
        Self {
            window: None,
            title,
            app: Arc::new(Mutex::new(None)),
            missed_resize: Arc::new(Mutex::new(None)),
        }
    }

    fn config_window(&mut self) {
        let window = self.window.as_mut().unwrap();
        window.set_title(self.title);
        if cfg!(not(target_arch = "wasm32")) {
            // 计算一个默认显示高度
            let height = 600 * window.scale_factor() as u32;
            let width = height;
            let _ = window.request_inner_size(PhysicalSize::new(width, height));
        }

        #[cfg(target_arch = "wasm32")]
        {
            let canvas = window.canvas().unwrap();

            // 将 canvas 添加到当前网页中
            web_sys::window()
                .and_then(|win| win.document())
                .map(|doc| {
                    let _ = canvas.set_attribute("id", "winit-canvas");
                    match doc.get_element_by_id("wgpu-app-container") {
                        Some(dst) => {
                            let _ = dst.append_child(canvas.as_ref());
                        }
                        None => {
                            let container = doc.create_element("div").unwrap();
                            let _ = container.set_attribute("id", "wgpu-app-container");
                            let _ = container.append_child(canvas.as_ref());

                            doc.body().map(|body| body.append_child(container.as_ref()));
                        }
                    };
                })
                .expect("无法将 canvas 添加到当前网页中");

            // 确保画布可以获得焦点
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex
            canvas.set_tab_index(0);

            // 设置画布获得焦点时不显示高亮轮廓
            let style = canvas.style();
            style.set_property("outline", "none").unwrap();
            canvas.focus().expect("画布无法获取焦点");
        }
    }

    /// 在提交渲染之前通知窗口系统。
    fn pre_present_notify(&self) {
        if let Some(window) = self.window.as_ref() {
            window.pre_present_notify();
        }
    }

    /// 请求重绘    
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl<A: WgpuAppAction + 'static> ApplicationHandler for WgpuAppHandler<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.app.as_ref().lock().is_some() {
            return;
        }

        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.window = Some(window.clone());
        self.config_window();

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let app = self.app.clone();
                let missed_resize = self.missed_resize.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let window_cloned = window.clone();

                    let wgpu_app = A::new(window).await;
                    let mut app = app.lock();
                    *app = Some(wgpu_app);

                    if let Some(resize) = *missed_resize.lock() {
                        app.as_mut().unwrap().set_window_size(resize);
                        window_cloned.request_redraw();
                    }
                });
            } else {
                let wgpu_app = pollster::block_on(A::new(window));
                self.app.lock().replace(wgpu_app);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut app = self.app.lock();
        if app.as_ref().is_none() {
            if let WindowEvent::Resized(physical_size) = event
                && physical_size.width > 0
                && physical_size.height > 0
            {
                let mut missed_resize = self.missed_resize.lock();
                *missed_resize = Some(physical_size);
            }
            return;
        }

        let app = app.as_mut().unwrap();
        match event {
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    log::info!("Window minimized!");
                } else {
                    log::info!("Window resized: {:?}", physical_size);

                    app.set_window_size(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                self.pre_present_notify();
                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => eprintln!("Surface is lost"),
                    Err(e) => eprintln!("{e:?}"),
                }

                self.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

pub fn run<A: WgpuAppAction + 'static>(title: &'static str) -> Result<(), impl std::error::Error> {
    crate::init_logger();

    let events_loop = EventLoop::new().unwrap();
    let mut app = WgpuAppHandler::<A>::new(title);
    events_loop.run_app(&mut app)
}
