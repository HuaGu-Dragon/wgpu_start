#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {}

pub fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let query_string = web_sys::window().unwrap().location().search().unwrap();
            let query_level: Option<log::LevelFilter> = parse_url_query_string(&query_string, "RUST_LOG")
                .and_then(|x| x.parse().ok());

            let base_level = query_level.unwrap_or(log::LevelFilter::Info);
            let wgpu_level = query_level.unwrap_or(log::LevelFilter::Error);

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
            env_logger::builder().filter_level(log::LevelFilter::Info).filter_module("wgpu_core", log::LevelFilter::Info).filter_module("wgpu_hal", log::LevelFilter::Error).filter_module("naga", log::LevelFilter::Error).parse_default_env().init();
        }
    }
}
