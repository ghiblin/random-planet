use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    Document, Element, Event, HtmlDialogElement, HtmlInputElement, MessageEvent, Worker,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::web::WindowAttributesExtWebSys;
use winit::window::{Window, WindowId};

use planet_core::geometry::mesh::Mesh;
use planet_core::presets::preset::Preset;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::steps::Steps;

use crate::controls::depth_slider;
use crate::controls::preset_select::parse_preset;
use crate::controls::seed_from_timestamp::seed_from_timestamp;
use crate::gpu::buffers::pack_frame;
use crate::gpu::render::Renderer;
use crate::scene::camera::Camera;
use crate::scene::growth_animation::GrowthAnimation;
use crate::worker::protocol::{StartRequest, WorkerMessage};

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_LINE_SENSITIVITY: f32 = 0.5;
const ZOOM_PIXEL_SENSITIVITY: f32 = 0.01;

/// Trunk's worker-asset pipeline emits the `generation_worker` bin target's loader
/// script under this name (`data-trunk rel="rust" data-type="worker"
/// data-bin="generation_worker"` in `index.html`) — confirmed/adjusted against Trunk's
/// actual build output during manual in-browser verification.
const GENERATION_WORKER_SCRIPT_URL: &str = "./generation_worker_loader.js";

/// The in-progress growth animation, shared between winit's event loop and the DOM
/// closures set up in `App::wire_controls`. `None` before the first generation.
type Frames = Rc<RefCell<Option<GrowthAnimation>>>;

/// Reads the browser's high-resolution clock, used to pace the growth animation by
/// wall-clock elapsed time rather than by redraw frequency. `None` (logged) if the
/// `Performance` API is unavailable, e.g. no `window`.
fn performance_now_ms() -> Option<f64> {
    let performance = web_sys::window().and_then(|window| window.performance());
    if performance.is_none() {
        log_error("failed to access Performance API");
    }
    performance.map(|performance| performance.now())
}

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<Option<Renderer>>>,
    camera: Camera,
    dragging: bool,
    last_cursor: Option<PhysicalPosition<f64>>,
    frames: Frames,
    worker: Option<Worker>,
    wireframe: Rc<RefCell<bool>>,
    flat_shading: Rc<RefCell<bool>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: Rc::new(RefCell::new(None)),
            camera: Camera::default(),
            dragging: false,
            last_cursor: None,
            frames: Rc::new(RefCell::new(None)),
            worker: None,
            wireframe: Rc::new(RefCell::new(false)),
            flat_shading: Rc::new(RefCell::new(false)),
        }
    }
}

fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

fn document() -> Option<Document> {
    let doc = web_sys::window().and_then(|window| window.document());
    if doc.is_none() {
        log_error("failed to access document");
    }
    doc
}

fn get_element(document: &Document, id: &str) -> Option<Element> {
    let element = document.get_element_by_id(id);
    if element.is_none() {
        log_error(&format!("missing expected element #{id}"));
    }
    element
}

fn get_typed_element<T: JsCast>(document: &Document, id: &str) -> Option<T> {
    let element = get_element(document, id)?;
    match element.dyn_into::<T>() {
        Ok(typed) => Some(typed),
        Err(_) => {
            log_error(&format!("element #{id} is not the expected type"));
            None
        }
    }
}

/// Syncs a toggle switch's checkbox to `checked` — the single place the keyboard
/// shortcut updates the on-screen switch, so it never drifts out of sync with a click
/// made directly on the switch itself.
fn sync_checkbox(document: &Document, id: &str, checked: bool) {
    if let Some(checkbox) = get_typed_element::<HtmlInputElement>(document, id) {
        checkbox.set_checked(checked);
    }
}

fn create_element(document: &Document, tag: &str) -> Option<Element> {
    match document.create_element(tag) {
        Ok(element) => Some(element),
        Err(_) => {
            log_error(&format!("failed to create <{tag}> element"));
            None
        }
    }
}

/// Shows `#error-modal` with `message`. Scoped to the one call site that needs it
/// (the startup empty-mesh construction, see `resumed`) — every other error branch
/// in this file keeps its existing console-log-and-return behavior.
fn show_error_modal(document: &Document, message: &str) {
    let Some(dialog) = get_typed_element::<HtmlDialogElement>(document, "error-modal") else {
        return;
    };
    let Some(message_element) = get_element(document, "error-modal-message") else {
        return;
    };
    message_element.set_text_content(Some(message));
    if dialog.show_modal().is_err() {
        log_error("failed to show error modal");
    }
}

fn populate_preset_group(document: &Document, preset_group: &Element) {
    for preset in Preset::ALL {
        let (Some(label), Some(radio), Some(image), Some(label_text), Some(description)) = (
            create_element(document, "label"),
            create_element(document, "input"),
            create_element(document, "img"),
            create_element(document, "span"),
            create_element(document, "span"),
        ) else {
            log_error("failed to create preset option elements");
            continue;
        };

        let _ = label.set_attribute("class", "preset-option");

        let _ = radio.set_attribute("type", "radio");
        let _ = radio.set_attribute("name", "preset");
        let _ = radio.set_attribute("id", &format!("preset-{}", preset.name()));
        let _ = radio.set_attribute("value", preset.name());
        if preset == Preset::default() {
            let _ = radio.set_attribute("checked", "");
        }

        let _ = image.set_attribute("class", "preset-image-placeholder");
        let _ = image.set_attribute("alt", &format!("{} preset preview", preset.name()));

        label_text.set_text_content(Some(preset.name()));
        let _ = label_text.set_attribute("class", "preset-label");

        description.set_text_content(Some(preset.description()));
        let _ = description.set_attribute("class", "preset-description");

        let _ = label.append_child(&radio);
        let _ = label.append_child(&image);
        let _ = label.append_child(&label_text);
        let _ = label.append_child(&description);
        let _ = preset_group.append_child(&label);
    }
}

fn configure_depth_slider(slider: &HtmlInputElement) {
    slider.set_min(&depth_slider::MIN_DEPTH.to_string());
    slider.set_max(&depth_slider::MAX_DEPTH.to_string());
    slider.set_value(&Steps::default().value().to_string());
}

fn create_generation_worker() -> Option<Worker> {
    // The loader Trunk emits for a worker asset (`generation_worker_loader.js`) uses
    // `importScripts(...)` to load the wasm-bindgen glue, which is only available to
    // classic workers — a module worker has no `importScripts` and would fail at
    // runtime, so this must stay the default (classic) `Worker::new`, not
    // `new_with_options` with `WorkerType::Module`.
    match Worker::new(GENERATION_WORKER_SCRIPT_URL) {
        Ok(worker) => Some(worker),
        Err(_) => {
            log_error("failed to create generation worker");
            None
        }
    }
}

fn set_start_button_disabled(document: &Document, disabled: bool) {
    let Some(start_button) = get_element(document, "start-button") else {
        return;
    };
    if disabled {
        let _ = start_button.set_attribute("disabled", "");
    } else {
        let _ = start_button.remove_attribute("disabled");
    }
}

/// Posts a `StartRequest` to the generation worker instead of computing anything on
/// the main thread — every Start click gets its own timestamp-derived seed, so there
/// is nothing to reuse from a previous generation. Resets `frames` to a fresh, empty
/// `GrowthAnimation` so the new generation's reveal starts clean, and disables
/// `#start-button` for the duration of the in-flight request (re-enabled by
/// `handle_worker_message` once a final frame or an error arrives).
fn generate(
    preset: Preset,
    depth: Steps,
    seed: Seed,
    worker: &Worker,
    frames: &Frames,
    document: &Document,
) {
    match frames.try_borrow_mut() {
        Ok(mut frames_ref) => *frames_ref = Some(GrowthAnimation::new()),
        Err(_) => {
            log_error("generate: frames already borrowed when resetting for new generation");
            return;
        }
    }

    set_start_button_disabled(document, true);

    let request = StartRequest {
        preset,
        depth,
        seed,
    };
    if worker.post_message(&request.to_js_value()).is_err() {
        log_error("failed to post StartRequest to generation worker");
    }
}

/// Handles one decoded `WorkerMessage` from the generation worker: pushes `Frame`s
/// into the growth animation (uploading to the GPU immediately if this is the very
/// first frame of the generation, matching `GrowthAnimation::push_frame`'s own
/// reveal-immediately rule), and re-enables `#start-button` once a final frame or an
/// error arrives.
fn handle_worker_message(
    message: WorkerMessage,
    frames: &Frames,
    renderer: &Rc<RefCell<Option<Renderer>>>,
    document: &Document,
    window: &Window,
) {
    match message {
        WorkerMessage::Frame { frame, is_final } => {
            let Some(now_ms) = performance_now_ms() else {
                return;
            };
            match frames.try_borrow_mut() {
                Ok(mut frames_ref) => {
                    let animation = frames_ref.get_or_insert_with(GrowthAnimation::new);
                    let was_empty = animation.current().is_none();
                    animation.push_frame(frame, now_ms);
                    if was_empty {
                        if let Some(current) = animation.current() {
                            match renderer.try_borrow_mut() {
                                Ok(mut renderer_ref) => {
                                    if let Some(renderer) = renderer_ref.as_mut() {
                                        renderer.set_mesh(current);
                                    }
                                }
                                Err(_) => log_error(
                                    "worker message: renderer already borrowed when pushing first frame",
                                ),
                            }
                        }
                    }
                }
                Err(_) => log_error("worker message: frames already borrowed"),
            }
            if is_final {
                set_start_button_disabled(document, false);
            }
            window.request_redraw();
        }
        WorkerMessage::PostprocessStage(_stage) => {
            // Hook for a future status label; not required to fix the reported freeze.
        }
        WorkerMessage::Error(message) => {
            log_error(&format!("generation worker error: {message}"));
            set_start_button_disabled(document, false);
        }
    }
}

impl App {
    fn wire_controls(&self) {
        let Some(document) = document() else {
            return;
        };

        if let Some(preset_group) = get_element(&document, "preset-group") {
            populate_preset_group(&document, &preset_group);
        }

        if let Some(depth_slider_el) =
            get_typed_element::<HtmlInputElement>(&document, "depth-slider")
        {
            configure_depth_slider(&depth_slider_el);
            if let Some(label) = get_element(&document, "depth-value") {
                label.set_text_content(Some(&depth_slider_el.value()));
            }

            let document_for_input = document.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                let (Some(slider), Some(label)) = (
                    get_typed_element::<HtmlInputElement>(&document_for_input, "depth-slider"),
                    get_element(&document_for_input, "depth-value"),
                ) else {
                    return;
                };
                label.set_text_content(Some(&slider.value()));
            });
            let _ = depth_slider_el
                .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        if let Some(start_button) = get_element(&document, "start-button") {
            let document_for_start = document.clone();
            let renderer = self.renderer.clone();
            let frames = self.frames.clone();
            let worker = self.worker.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                let document = &document_for_start;

                let Some(preset_radio) = document
                    .query_selector("input[name=preset]:checked")
                    .ok()
                    .flatten()
                    .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
                else {
                    log_error("no preset radio is checked");
                    return;
                };
                let Some(preset) = parse_preset(&preset_radio.value()) else {
                    log_error(&format!(
                        "unrecognized preset value: {}",
                        preset_radio.value()
                    ));
                    return;
                };

                let Some(depth_slider_el) =
                    get_typed_element::<HtmlInputElement>(document, "depth-slider")
                else {
                    return;
                };
                let depth = match depth_slider::parse_depth(&depth_slider_el.value()) {
                    Ok(depth) => depth,
                    Err(error) => {
                        log_error(&format!("invalid depth-slider value: {error}"));
                        return;
                    }
                };

                let seed = seed_from_timestamp(js_sys::Date::now());

                let renderer_ready = match renderer.try_borrow() {
                    Ok(guard) => guard.is_some(),
                    Err(_) => {
                        log_error("start-button: renderer already borrowed during readiness check");
                        false
                    }
                };
                if !renderer_ready {
                    log_error("renderer not ready yet; ignoring Start click");
                    return;
                }

                let Some(worker) = worker.as_ref() else {
                    log_error("generation worker not ready yet; ignoring Start click");
                    return;
                };
                generate(preset, depth, seed, worker, &frames, document);
            });
            let _ = start_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        if let Some(wireframe_toggle) =
            get_typed_element::<HtmlInputElement>(&document, "wireframe-toggle")
        {
            let wireframe = self.wireframe.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Some(checkbox) = event
                    .target()
                    .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                else {
                    return;
                };
                *wireframe.borrow_mut() = checkbox.checked();
            });
            let _ = wireframe_toggle
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        if let Some(flat_shading_toggle) =
            get_typed_element::<HtmlInputElement>(&document, "flat-shading-toggle")
        {
            let flat_shading = self.flat_shading.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Some(checkbox) = event
                    .target()
                    .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                else {
                    return;
                };
                *flat_shading.borrow_mut() = checkbox.checked();
            });
            let _ = flat_shading_toggle
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        if let Some(dismiss) = get_element(&document, "error-modal-dismiss") {
            let document_for_dismiss = document.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                if let Some(dialog) =
                    get_typed_element::<HtmlDialogElement>(&document_for_dismiss, "error-modal")
                {
                    dialog.close();
                }
            });
            let _ =
                dismiss.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes().with_append(true);
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                log_error(&format!("failed to create window: {error}"));
                return;
            }
        };
        self.window = Some(window.clone());
        window.request_redraw();

        self.worker = create_generation_worker();
        if let Some(worker) = &self.worker {
            let frames = self.frames.clone();
            let renderer = self.renderer.clone();
            let window_for_messages = window.clone();
            let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                let Some(document) = document() else {
                    return;
                };
                let Some(message) = WorkerMessage::from_js_value(&event.data()) else {
                    log_error("failed to decode a message from the generation worker");
                    return;
                };
                handle_worker_message(message, &frames, &renderer, &document, &window_for_messages);
            });
            worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
        }

        self.wire_controls();

        let empty_mesh = match Mesh::new(vec![], vec![]) {
            Ok(mesh) => mesh,
            Err(error) => {
                if let Some(document) = document() {
                    show_error_modal(&document, &format!("Failed to initialize: {error}"));
                }
                return;
            }
        };
        let initial_frame = pack_frame(&empty_mesh, &[]);

        let renderer_slot = self.renderer.clone();
        let size_probe = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Renderer::new(window, &initial_frame).await {
                Ok(mut renderer) => {
                    // The canvas may have been resized by the browser (e.g. once its
                    // ResizeObserver reports the real layout size) while the adapter/device
                    // were still negotiating; resync against the window's current size.
                    renderer.resize(size_probe.inner_size());
                    match renderer_slot.try_borrow_mut() {
                        Ok(mut slot) => *slot = Some(renderer),
                        Err(_) => log_error("renderer setup: renderer_slot already borrowed"),
                    }
                    if let Some(document) = document() {
                        if let Some(start_button) = get_element(&document, "start-button") {
                            let _ = start_button.remove_attribute("disabled");
                        }
                    }
                }
                Err(error) => {
                    log_error(&format!("failed to create renderer: {error}"));
                }
            }
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => match self.renderer.try_borrow_mut() {
                Ok(mut renderer_ref) => {
                    if let Some(renderer) = renderer_ref.as_mut() {
                        renderer.resize(size);
                    }
                }
                Err(_) => log_error("resize: renderer already borrowed"),
            },
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = state == ElementState::Pressed;
                if !self.dragging {
                    self.last_cursor = None;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.dragging {
                    if let Some(last) = self.last_cursor {
                        let delta_yaw = (position.x - last.x) as f32 * ORBIT_SENSITIVITY;
                        let delta_pitch = -(position.y - last.y) as f32 * ORBIT_SENSITIVITY;
                        self.camera.orbit(delta_yaw, delta_pitch);
                    }
                    self.last_cursor = Some(position);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * ZOOM_LINE_SENSITIVITY,
                    MouseScrollDelta::PixelDelta(position) => {
                        position.y as f32 * ZOOM_PIXEL_SENSITIVITY
                    }
                };
                self.camera.zoom(-scroll);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && !event.repeat {
                    let toggled = match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            let mut flag = self.wireframe.borrow_mut();
                            *flag = !*flag;
                            Some(("wireframe-toggle", *flag))
                        }
                        PhysicalKey::Code(KeyCode::KeyF) => {
                            let mut flag = self.flat_shading.borrow_mut();
                            *flag = !*flag;
                            Some(("flat-shading-toggle", *flag))
                        }
                        _ => None,
                    };
                    if let Some((id, checked)) = toggled {
                        if let Some(document) = document() {
                            sync_checkbox(&document, id, checked);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(now_ms) = performance_now_ms() {
                    match self.frames.try_borrow_mut() {
                        Ok(mut frames) => {
                            if let Some(animation) = frames.as_mut() {
                                if animation.tick(now_ms) {
                                    match self.renderer.try_borrow_mut() {
                                        Ok(mut renderer_ref) => {
                                            if let Some(renderer) = renderer_ref.as_mut() {
                                                if let Some(frame) = animation.current() {
                                                    renderer.set_mesh(frame);
                                                }
                                            }
                                        }
                                        Err(_) => log_error(
                                            "redraw: renderer already borrowed while advancing frame",
                                        ),
                                    }
                                }
                            }
                        }
                        Err(_) => log_error("redraw: frames already borrowed"),
                    }
                }
                match self.renderer.try_borrow() {
                    Ok(renderer_ref) => {
                        if let Some(renderer) = renderer_ref.as_ref() {
                            renderer.render(
                                &self.camera,
                                *self.wireframe.borrow(),
                                *self.flat_shading.borrow(),
                            );
                        }
                    }
                    Err(_) => log_error("redraw: renderer already borrowed while rendering"),
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
