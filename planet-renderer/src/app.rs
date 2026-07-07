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

use planet_core::icosahedron::icosahedron;
use planet_core::uniform_red_split::UniformRedSplit;

use crate::camera::Camera;
use crate::render::Renderer;
use crate::subdivision_stepper::SubdivisionStepper;

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_LINE_SENSITIVITY: f32 = 0.5;
const ZOOM_PIXEL_SENSITIVITY: f32 = 0.01;
// Temporary hardcoded value until 007-planet-presets wires up the depth slider.
const MAX_SUBDIVISION_DEPTH: u32 = 3;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<Option<Renderer>>>,
    camera: Camera,
    dragging: bool,
    last_cursor: Option<PhysicalPosition<f64>>,
    stepper: Option<SubdivisionStepper>,
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
            stepper: None,
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

        let base_mesh = match icosahedron() {
            Ok(mesh) => mesh,
            Err(error) => {
                web_sys::console::error_1(
                    &format!("failed to construct icosahedron: {error}").into(),
                );
                return;
            }
        };
        let stepper = SubdivisionStepper::new(base_mesh, MAX_SUBDIVISION_DEPTH);
        let initial_mesh = stepper.mesh().clone();
        self.stepper = Some(stepper);

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
                        PhysicalKey::Code(KeyCode::Space) => {
                            let advanced = self
                                .stepper
                                .as_mut()
                                .map(|stepper| stepper.step(&mut UniformRedSplit).unwrap_or(false))
                                .unwrap_or(false);
                            if advanced {
                                if let (Some(stepper), Some(renderer)) =
                                    (&self.stepper, self.renderer.borrow_mut().as_mut())
                                {
                                    renderer.set_mesh(stepper.mesh());
                                }
                                if let Some(window) = &self.window {
                                    window.request_redraw();
                                }
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            self.wireframe = !self.wireframe;
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
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
