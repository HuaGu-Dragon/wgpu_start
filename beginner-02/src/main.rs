use app_surface::AppSurface;
use winit::dpi::PhysicalSize;

fn main() {
    println!("Hello, world!");
}

struct WgpuApp {
    app: AppSurface,
    size: PhysicalSize<u32>,
    size_changed: bool,
    render_pipeline: wgpu::RenderPipeline,
}
