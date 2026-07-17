#[cfg(target_arch = "wasm32")]
pub mod app;
pub mod controls;
pub mod gpu;
pub mod scene;
pub mod worker;

/// `#[wasm_bindgen(start)]` runs automatically whenever any wasm module containing
/// this exported function is instantiated — including inside the `generation_worker`
/// bin's own wasm module, since it links this same `planet-renderer` lib crate. A
/// `Window`/DOM-driven winit event loop cannot exist inside a Worker (no `Window`
/// global there at all), so bail out immediately unless a real browser `window` is
/// present, which is only true for the main-thread bundle this function is meant for.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    use winit::platform::web::EventLoopExtWebSys;

    if web_sys::window().is_none() {
        return Ok(());
    }

    console_error_panic_hook::set_once();

    let event_loop = winit::event_loop::EventLoop::new()
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.spawn_app(app::App::default());
    Ok(())
}
