use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::web::WindowAttributesExtWebSys;
use winit::window::{Window, WindowId};

use planet_core::geometry::mesh::Mesh;
use planet_core::processor::vertex_scramble::scramble_vertices;
use planet_core::processor::vertex_scramble_range::VertexScrambleRange;
use planet_core::subdivision::elevation_noise_range::ElevationNoiseRange;
use planet_core::subdivision::min_edge_length::MinEdgeLength;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::split_point_variance::SplitPointVariance;
use planet_core::subdivision::subdivide::subdivide;
use planet_core::subdivision::subdivision_args::SubdivisionArgs;
use planet_core::subdivision::subdivision_mode::SubdivisionMode;

use crate::gpu::render::Renderer;
use crate::scene::camera::Camera;

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_LINE_SENSITIVITY: f32 = 0.5;
const ZOOM_PIXEL_SENSITIVITY: f32 = 0.01;
const DEMO_SEED: u64 = 42;
const DEMO_SCRAMBLE_SEED: u64 = 43;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<Option<Renderer>>>,
    camera: Camera,
    dragging: bool,
    last_cursor: Option<PhysicalPosition<f64>>,
    frames: Vec<Mesh>,
    current_frame: usize,
    wireframe: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: Rc::new(RefCell::new(None)),
            camera: Camera::default(),
            dragging: false,
            last_cursor: None,
            frames: Vec::new(),
            current_frame: 0,
            wireframe: false,
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
                web_sys::console::error_1(&format!("failed to create window: {error}").into());
                return;
            }
        };
        self.window = Some(window.clone());
        window.request_redraw();

        let base_mesh = match Mesh::icosahedron() {
            Ok(mesh) => mesh,
            Err(error) => {
                web_sys::console::error_1(
                    &format!("failed to construct icosahedron: {error}").into(),
                );
                return;
            }
        };
        let base_mesh = match scramble_vertices(
            &base_mesh,
            Seed::from(DEMO_SCRAMBLE_SEED),
            VertexScrambleRange::default(),
        ) {
            Ok(mesh) => mesh,
            Err(error) => {
                web_sys::console::error_1(&format!("failed to scramble vertices: {error}").into());
                return;
            }
        };

        let collected_frames = Rc::new(RefCell::new(vec![base_mesh.clone()]));
        let frame_collector = collected_frames.clone();
        let update_cb: Box<dyn FnMut(&Mesh, usize)> = Box::new(move |mesh, _round| {
            frame_collector.borrow_mut().push(mesh.clone());
        });
        let args = SubdivisionArgs::new(
            None,
            Some(SubdivisionMode::RedGreenSplit {
                seed: Seed::from(DEMO_SEED),
                elevation_noise_range: ElevationNoiseRange::default(),
                min_edge_length: MinEdgeLength::default(),
                split_point_variance: SplitPointVariance::default(),
            }),
            Some(update_cb),
        );
        if let Err(error) = subdivide(&base_mesh, args) {
            web_sys::console::error_1(&format!("failed to subdivide icosahedron: {error}").into());
            return;
        }
        self.frames = match Rc::try_unwrap(collected_frames) {
            Ok(frames) => frames.into_inner(),
            Err(_) => {
                web_sys::console::error_1(
                    &"failed to collect subdivision frames: update_cb outlived subdivide".into(),
                );
                return;
            }
        };
        self.current_frame = 0;
        let initial_mesh = self.frames[0].clone();

        let renderer_slot = self.renderer.clone();
        let size_probe = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Renderer::new(window, &initial_mesh).await {
                Ok(mut renderer) => {
                    // The canvas may have been resized by the browser (e.g. once its
                    // ResizeObserver reports the real layout size) while the adapter/device
                    // were still negotiating; resync against the window's current size.
                    renderer.resize(size_probe.inner_size());
                    *renderer_slot.borrow_mut() = Some(renderer);
                }
                Err(error) => {
                    web_sys::console::error_1(
                        &format!("failed to create renderer: {error}").into(),
                    );
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
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    renderer.resize(size);
                }
            }
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
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.current_frame + 1 < self.frames.len() {
                    self.current_frame += 1;
                    if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                        renderer.set_mesh(&self.frames[self.current_frame]);
                    }
                }
                if let Some(renderer) = self.renderer.borrow().as_ref() {
                    renderer.render(&self.camera, self.wireframe);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
