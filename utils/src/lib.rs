use std::sync::Arc;
pub mod framework;

use winit::window::Window;

pub fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            // let query_string = web_sys::window().unwrap().location().search().unwrap();
            // let query_level: Option<log::LevelFilter> = parse_url_query_string(&query_string, "RUST_LOG")
            //     .and_then(|x| x.parse().ok());

            // let base_level = query_level.unwrap_or(log::LevelFilter::Info);
            // let wgpu_level = query_level.unwrap_or(log::LevelFilter::Error);

            // fern::Dispatch::new()
            //     .level(base_level)
            //     .level_for("wgpu_core", wgpu_level)
            //     .level_for("wgpu_hal", wgpu_level)
            //     .level_for("naga", wgpu_level)
            //     .chain(fern::Output::call(console_log::log))
            //     .apply()
            //     .unwrap();
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        } else {
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                .filter_module("wgpu_core", log::LevelFilter::Info)
                .filter_module("wgpu_hal", log::LevelFilter::Error)
                .filter_module("naga", log::LevelFilter::Error)
                .parse_default_env()
                .init();

        }
    }
}

#[allow(unused)]
pub fn init(window: Arc<Window>) {
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas().unwrap();

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
            .expect("cannot add canvas to current webpage");

        canvas.set_tab_index(0);

        let style = canvas.style();
        style.set_property("outline", "none").unwrap();
        canvas.focus().expect("canvas cannot gain focus");
    }
}
