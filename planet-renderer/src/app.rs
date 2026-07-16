use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Element, Event, HtmlDialogElement, HtmlInputElement};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::web::WindowAttributesExtWebSys;
use winit::window::{Window, WindowId};

use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;
use planet_core::planets::planet::{GenerationProgress, Planet};
use planet_core::presets::preset::Preset;
use planet_core::processor::finalize_normals::finalize_normals;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::steps::Steps;

use crate::controls::depth_slider;
use crate::controls::preset_select::parse_preset;
use crate::controls::seed_from_timestamp::seed_from_timestamp;
use crate::gpu::render::Renderer;
use crate::scene::camera::Camera;

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_LINE_SENSITIVITY: f32 = 0.5;
const ZOOM_PIXEL_SENSITIVITY: f32 = 0.01;

/// The current growth-animation frame list and playback position, shared between
/// winit's event loop and the DOM closures set up in `App::wire_controls`.
type Frames = Rc<RefCell<(Vec<(Mesh, Vec<Rgb>)>, usize)>>;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<Option<Renderer>>>,
    camera: Camera,
    dragging: bool,
    last_cursor: Option<PhysicalPosition<f64>>,
    frames: Frames,
    wireframe: bool,
    flat_shading: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: Rc::new(RefCell::new(None)),
            camera: Camera::default(),
            dragging: false,
            last_cursor: None,
            frames: Rc::new(RefCell::new((Vec::new(), 0))),
            wireframe: false,
            flat_shading: false,
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

/// Builds and subdivides a fresh `Planet` from `(preset, depth, seed)`, always from
/// scratch — every Start click gets its own timestamp-derived seed, so there is
/// nothing to reuse from a previous generation. Collects the per-round growth
/// animation into `frames` (colored via the preset's own `ColorGradient`, since
/// that's a pure function of a vertex's radius, valid for every intermediate round;
/// normals are finalized the same way per frame so the growth animation renders with
/// smooth shading too), then swaps the last collected frame for `Planet::subdivide`'s
/// true, fully post-processed result before handing the first frame to the renderer.
fn generate(
    preset: Preset,
    depth: Steps,
    seed: Seed,
    renderer: &Rc<RefCell<Option<Renderer>>>,
    frames: &Frames,
    window: &Window,
) {
    let planet = match Planet::builder()
        .with_preset(preset)
        .with_seed(seed)
        .build()
    {
        Ok(planet) => planet,
        Err(error) => {
            log_error(&format!("failed to create planet: {error}"));
            return;
        }
    };

    let params = preset.params();
    let collected_frames = Rc::new(RefCell::new(Vec::new()));
    let frame_collector = collected_frames.clone();
    let on_progress: GenerationProgress = Box::new(move |mesh, _round| {
        let colors = mesh
            .vertices()
            .iter()
            .map(|vertex| params.color_gradient().sample(vertex.position.length()))
            .collect();
        frame_collector
            .borrow_mut()
            .push((finalize_normals(mesh), colors));
    });

    let subdivided = match planet.subdivide(depth, Some(on_progress)) {
        Ok(subdivided) => subdivided,
        Err(error) => {
            log_error(&format!("failed to subdivide planet: {error}"));
            return;
        }
    };

    let mut new_frames = match Rc::try_unwrap(collected_frames) {
        Ok(cell) => cell.into_inner(),
        Err(_) => {
            log_error("failed to collect generation frames: on_progress outlived generate");
            return;
        }
    };

    if let Some(last) = new_frames.last_mut() {
        *last = (subdivided.mesh().clone(), subdivided.colors().to_vec());
    }

    match frames.try_borrow_mut() {
        Ok(mut frames_ref) => *frames_ref = (new_frames, 0),
        Err(_) => {
            log_error("generate: frames already borrowed when storing new generation");
            return;
        }
    }

    match renderer.try_borrow_mut() {
        Ok(mut renderer_ref) => {
            if let Some(renderer) = renderer_ref.as_mut() {
                match frames.try_borrow() {
                    Ok(frames_ref) => {
                        if let Some((mesh, colors)) = frames_ref.0.first() {
                            renderer.set_mesh(mesh, colors);
                        }
                    }
                    Err(_) => {
                        log_error("generate: frames already borrowed when reading first frame")
                    }
                }
            }
        }
        Err(_) => log_error("generate: renderer already borrowed when pushing first frame"),
    }

    window.request_redraw();
}

impl App {
    fn wire_controls(&self, window: &Arc<Window>) {
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
            let window = window.clone();
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

                generate(preset, depth, seed, &renderer, &frames, &window);

                if let Some(controls) = get_element(document, "controls") {
                    let _ = controls.set_attribute("hidden", "");
                }
                if let Some(change_settings) = get_element(document, "change-settings-button") {
                    let _ = change_settings.remove_attribute("hidden");
                }
            });
            let _ = start_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        if let Some(change_settings) = get_element(&document, "change-settings-button") {
            let document_for_change = document.clone();
            let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                if let Some(controls) = get_element(&document_for_change, "controls") {
                    let _ = controls.remove_attribute("hidden");
                }
                if let Some(change_settings) =
                    get_element(&document_for_change, "change-settings-button")
                {
                    let _ = change_settings.set_attribute("hidden", "");
                }
            });
            let _ = change_settings
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
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
        self.wire_controls(&window);

        let empty_mesh = match Mesh::new(vec![], vec![]) {
            Ok(mesh) => mesh,
            Err(error) => {
                if let Some(document) = document() {
                    show_error_modal(&document, &format!("Failed to initialize: {error}"));
                }
                return;
            }
        };

        let renderer_slot = self.renderer.clone();
        let size_probe = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Renderer::new(window, &empty_mesh, &[]).await {
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
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            self.wireframe = !self.wireframe;
                        }
                        PhysicalKey::Code(KeyCode::KeyF) => {
                            self.flat_shading = !self.flat_shading;
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                match self.frames.try_borrow_mut() {
                    Ok(mut frames) => {
                        let (frame_list, current_frame) = &mut *frames;
                        if *current_frame + 1 < frame_list.len() {
                            *current_frame += 1;
                            match self.renderer.try_borrow_mut() {
                                Ok(mut renderer_ref) => {
                                    if let Some(renderer) = renderer_ref.as_mut() {
                                        let (mesh, colors) = &frame_list[*current_frame];
                                        renderer.set_mesh(mesh, colors);
                                    }
                                }
                                Err(_) => log_error(
                                    "redraw: renderer already borrowed while advancing frame",
                                ),
                            }
                        }
                    }
                    Err(_) => log_error("redraw: frames already borrowed"),
                }
                match self.renderer.try_borrow() {
                    Ok(renderer_ref) => {
                        if let Some(renderer) = renderer_ref.as_ref() {
                            renderer.render(&self.camera, self.wireframe, self.flat_shading);
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
