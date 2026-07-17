//! The Web Worker entry point for planet generation. Runs `Planet::builder()...build()`
//! and `.subdivide(...)` off the main thread so `App` (the main-thread entry point in
//! `app.rs`) never blocks, regardless of subdivision depth.
//!
//! `fn main()` stays an empty, unconditional no-op on every target — Rust's own
//! generated entry-point mechanics own that symbol, and `wasm32-unknown-unknown` has
//! no OS-level process convention that would call it anyway. The real setup lives in
//! `wasm_start`, `#[wasm_bindgen(start)]`-tagged so the generated JS glue invokes it
//! once the module instantiates (the same mechanism `lib.rs`'s own `start()` uses for
//! the main-thread bundle) — gated so it only ever references a wasm-bindgen/web-sys
//! API on the `wasm32` target, keeping `cargo test --workspace`/`cargo build` (native)
//! trivially compiling this binary.
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn wasm_start() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

    use planet_renderer::worker::protocol::StartRequest;

    console_error_panic_hook::set_once();

    let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
    let global_for_closure = global.clone();
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        match StartRequest::from_js_value(&event.data()) {
            Some(request) => wasm::run_generation(&global_for_closure, request),
            None => wasm::post_error(&global_for_closure, "failed to decode StartRequest"),
        }
    });
    global.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use planet_core::planets::planet::{GenerationProgress, Planet, PostprocessProgress};
    use planet_core::processor::finalize_normals::finalize_normals;
    use web_sys::DedicatedWorkerGlobalScope;

    use planet_renderer::gpu::buffers::pack_frame;
    use planet_renderer::worker::protocol::{StartRequest, WorkerMessage};

    pub(super) fn post_error(global: &DedicatedWorkerGlobalScope, message: &str) {
        let worker_message = WorkerMessage::Error(message.to_string());
        let _ = global.post_message(&worker_message.to_js_value());
    }

    /// Runs one full generation synchronously — this blocks the *worker's* thread for
    /// its entire duration, which is fine, since it is never the main thread. Every
    /// intermediate subdivision round is packed and posted back immediately as a
    /// non-final `Frame`, except the very last round: its raw, unprocessed mesh is
    /// superseded by `Planet::subdivide`'s true, fully post-processed result before
    /// ever being posted, mirroring the pre-worker synchronous code's own "swap the
    /// last collected frame for the final result" behavior.
    pub(super) fn run_generation(global: &DedicatedWorkerGlobalScope, request: StartRequest) {
        let planet = match Planet::builder()
            .with_preset(request.preset)
            .with_seed(request.seed)
            .build()
        {
            Ok(planet) => planet,
            Err(error) => {
                post_error(global, &format!("failed to create planet: {error}"));
                return;
            }
        };

        let params = request.preset.params();
        let last_round = request.depth.value();
        let global_for_progress = global.clone();
        let on_progress: GenerationProgress = Box::new(move |mesh, round| {
            if round == last_round {
                return;
            }
            let colors: Vec<_> = mesh
                .vertices()
                .iter()
                .map(|vertex| params.color_gradient().sample(vertex.position.length()))
                .collect();
            let frame = pack_frame(&finalize_normals(mesh), &colors);
            let message = WorkerMessage::Frame {
                frame,
                is_final: false,
            };
            let _ = global_for_progress.post_message(&message.to_js_value());
        });

        let global_for_postprocess = global.clone();
        let on_postprocess: PostprocessProgress = Box::new(move |stage| {
            let message = WorkerMessage::PostprocessStage(stage);
            let _ = global_for_postprocess.post_message(&message.to_js_value());
        });

        let subdivided =
            match planet.subdivide(request.depth, Some(on_progress), Some(on_postprocess)) {
                Ok(subdivided) => subdivided,
                Err(error) => {
                    post_error(global, &format!("failed to subdivide planet: {error}"));
                    return;
                }
            };

        let frame = pack_frame(subdivided.mesh(), subdivided.colors());
        let message = WorkerMessage::Frame {
            frame,
            is_final: true,
        };
        let _ = global.post_message(&message.to_js_value());
    }
}
