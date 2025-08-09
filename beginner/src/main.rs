use std::sync::Arc;

use parking_lot::Mutex;
use winit::{application::ApplicationHandler, event, keyboard::PhysicalKey, window::Window};

fn main() {
    utils::init_logger();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut app = WgpuAppHandler::default();
    event_loop.run_app(&mut app).unwrap();
}

struct WgpuApp {
    #[allow(unused)]
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    size_changed: bool,
    clear_color: wgpu::Color,
}

impl WgpuApp {
    async fn new(window: Arc<Window>) -> Self {
        utils::init(window.clone());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            size_changed: false,
            clear_color,
        }
    }

    fn set_windows_size(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if self.size != size {
            self.size = size;
            self.size_changed = true;
        }
    }

    fn resize(&mut self) {
        if self.size_changed {
            self.size_changed = false;
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn keyboard_input(&mut self, event: &winit::event::KeyEvent) {
        if event.physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::Enter) {
            self.clear_color = if event.state == event::ElementState::Pressed {
                wgpu::Color::BLACK
            } else {
                wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        self.resize();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default)]
struct WgpuAppHandler {
    app: Arc<Mutex<Option<WgpuApp>>>,
}

impl ApplicationHandler for WgpuAppHandler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.app.as_ref().lock().is_some() {
            return;
        }

        let window_attributes = Window::default_attributes().with_title("WGPU App");
        let window = event_loop.create_window(window_attributes).unwrap();

        let wgpu_app: WgpuApp = pollster::block_on(WgpuApp::new(Arc::new(window)));

        self.app.lock().replace(wgpu_app);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut app = self.app.lock();

        let app = app.as_mut().unwrap();
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(size) => app.set_windows_size(size),
            winit::event::WindowEvent::RedrawRequested => {
                app.window.pre_present_notify();

                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => eprintln!("Surface lost, resizing..."),
                    Err(e) => eprintln!("Surface error: {:?}", e),
                }

                app.window.request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => app.keyboard_input(&event),
            _ => {}
        }
    }
}
