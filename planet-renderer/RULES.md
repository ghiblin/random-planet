# planet-renderer — Rules

## Allowed
- `wgpu`, `winit`, `wasm-bindgen`, `web-sys`
- Depends on `planet-core`

## Not allowed
- Domain/generation logic that belongs in `planet-core` (e.g. no subdivision math inline in render code)
