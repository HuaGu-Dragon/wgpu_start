use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use parking_lot::Mutex;
use wgpu::util::DeviceExt;
use winit::{application::ApplicationHandler, event, keyboard::PhysicalKey, window::Window};

fn main() {
    utils::init_logger();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut app = WgpuAppHandler::default();
    event_loop.run_app(&mut app).unwrap();
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

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
    render_pipeline: wgpu::RenderPipeline,
    use_color: bool,
    other_pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                compilation_options: Default::default(),
                entry_point: Some("vs_main"), // 1.
                buffers: &[],                 // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                compilation_options: Default::default(),
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // 将此设置为 Fill 以外的任何值都要需要开启 Feature::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // 需要开启 Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // 需要开启 Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Challenge Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("challenge.wgsl").into()),
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let other_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: core::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x4
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            cache: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            size_changed: false,
            clear_color,
            render_pipeline,
            other_pipeline,
            use_color: true,
            buffer,
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
            self.use_color = event.state == event::ElementState::Released;
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            render_pass.set_vertex_buffer(0, self.buffer.slice(..));
            render_pass.set_pipeline(if self.use_color {
                &self.render_pipeline
            } else {
                &self.other_pipeline
            });
            render_pass.draw(0..3, 0..1);
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
            winit::event::WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    log::info!("Window minimized!");
                } else {
                    log::info!("Window resized: {:?}", size);

                    app.set_windows_size(size);
                }
            }
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
