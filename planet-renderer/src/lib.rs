#[cfg(target_arch = "wasm32")]
pub mod app;
pub mod buffers;
pub mod camera;
pub mod render;
pub mod uniforms;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = winit::event_loop::EventLoop::new()
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.spawn_app(app::App::default());
    Ok(())
}
