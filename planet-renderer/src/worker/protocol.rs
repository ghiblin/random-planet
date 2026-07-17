use planet_core::planets::postprocess_stage::PostprocessStage;
use planet_core::presets::preset::Preset;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::steps::Steps;

use crate::gpu::buffers::PackedFrame;

/// Sent from the main thread to the generation worker to start one generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartRequest {
    pub preset: Preset,
    pub depth: Steps,
    pub seed: Seed,
}

/// Sent from the generation worker back to the main thread.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerMessage {
    Frame { frame: PackedFrame, is_final: bool },
    PostprocessStage(PostprocessStage),
    Error(String),
}

/// Thin, browser-only encode/decode between the plain types above and the
/// `postMessage`/`onmessage` `JsValue` boundary. Exempt from the Iron Law as thin
/// wiring, same class as `app.rs`'s DOM glue — the plain types above carry the real,
/// natively-testable shape of a message.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use js_sys::{Object, Reflect, Uint8Array};
    use wasm_bindgen::JsValue;

    impl StartRequest {
        pub fn to_js_value(self) -> JsValue {
            let object = Object::new();
            let _ = Reflect::set(&object, &"preset".into(), &self.preset.name().into());
            let _ = Reflect::set(
                &object,
                &"depth".into(),
                &(self.depth.value() as f64).into(),
            );
            let _ = Reflect::set(&object, &"seed".into(), &(self.seed.value() as f64).into());
            object.into()
        }

        pub fn from_js_value(value: &JsValue) -> Option<StartRequest> {
            let preset_name = Reflect::get(value, &"preset".into()).ok()?.as_string()?;
            let preset = crate::controls::preset_select::parse_preset(&preset_name)?;
            let depth = Reflect::get(value, &"depth".into()).ok()?.as_f64()?;
            let depth = Steps::new(depth as usize).ok()?;
            let seed = Reflect::get(value, &"seed".into()).ok()?.as_f64()?;
            Some(StartRequest {
                preset,
                depth,
                seed: Seed::from(seed as u64),
            })
        }
    }

    fn bytes_to_js(bytes: &[u8]) -> JsValue {
        Uint8Array::from(bytes).into()
    }

    fn js_to_bytes(value: &JsValue) -> Option<Vec<u8>> {
        Some(Uint8Array::new(value).to_vec())
    }

    fn frame_to_js(frame: &PackedFrame) -> JsValue {
        let object = Object::new();
        let _ = Reflect::set(
            &object,
            &"vertexBytesSmooth".into(),
            &bytes_to_js(&frame.vertex_bytes_smooth),
        );
        let _ = Reflect::set(
            &object,
            &"vertexBytesFlat".into(),
            &bytes_to_js(&frame.vertex_bytes_flat),
        );
        let _ = Reflect::set(
            &object,
            &"indexBytes".into(),
            &bytes_to_js(&frame.index_bytes),
        );
        let _ = Reflect::set(
            &object,
            &"lineIndexBytes".into(),
            &bytes_to_js(&frame.line_index_bytes),
        );
        object.into()
    }

    fn frame_from_js(value: &JsValue) -> Option<PackedFrame> {
        Some(PackedFrame {
            vertex_bytes_smooth: js_to_bytes(
                &Reflect::get(value, &"vertexBytesSmooth".into()).ok()?,
            )?,
            vertex_bytes_flat: js_to_bytes(&Reflect::get(value, &"vertexBytesFlat".into()).ok()?)?,
            index_bytes: js_to_bytes(&Reflect::get(value, &"indexBytes".into()).ok()?)?,
            line_index_bytes: js_to_bytes(&Reflect::get(value, &"lineIndexBytes".into()).ok()?)?,
        })
    }

    impl WorkerMessage {
        pub fn to_js_value(&self) -> JsValue {
            let object = Object::new();
            match self {
                WorkerMessage::Frame { frame, is_final } => {
                    let _ = Reflect::set(&object, &"type".into(), &"frame".into());
                    let _ = Reflect::set(&object, &"frame".into(), &frame_to_js(frame));
                    let _ = Reflect::set(&object, &"isFinal".into(), &(*is_final).into());
                }
                WorkerMessage::PostprocessStage(stage) => {
                    let _ = Reflect::set(&object, &"type".into(), &"postprocessStage".into());
                    let stage_name = match stage {
                        PostprocessStage::TerrainNoise => "TerrainNoise",
                        PostprocessStage::OceanQuota => "OceanQuota",
                    };
                    let _ = Reflect::set(&object, &"stage".into(), &stage_name.into());
                }
                WorkerMessage::Error(message) => {
                    let _ = Reflect::set(&object, &"type".into(), &"error".into());
                    let _ = Reflect::set(&object, &"message".into(), &message.as_str().into());
                }
            }
            object.into()
        }

        pub fn from_js_value(value: &JsValue) -> Option<WorkerMessage> {
            let message_type = Reflect::get(value, &"type".into()).ok()?.as_string()?;
            match message_type.as_str() {
                "frame" => {
                    let frame = frame_from_js(&Reflect::get(value, &"frame".into()).ok()?)?;
                    let is_final = Reflect::get(value, &"isFinal".into()).ok()?.as_bool()?;
                    Some(WorkerMessage::Frame { frame, is_final })
                }
                "postprocessStage" => {
                    let stage_name = Reflect::get(value, &"stage".into()).ok()?.as_string()?;
                    let stage = match stage_name.as_str() {
                        "TerrainNoise" => PostprocessStage::TerrainNoise,
                        "OceanQuota" => PostprocessStage::OceanQuota,
                        _ => return None,
                    };
                    Some(WorkerMessage::PostprocessStage(stage))
                }
                "error" => {
                    let message = Reflect::get(value, &"message".into()).ok()?.as_string()?;
                    Some(WorkerMessage::Error(message))
                }
                _ => None,
            }
        }
    }
}
