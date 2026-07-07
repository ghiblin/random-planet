# planet-core — Rules

## Allowed
- Pure computation (domain types, math, algorithms)
- `rand`, `rand_pcg`

## Not allowed
- `wgpu`, `winit`, `wasm-bindgen`, `web-sys`
- `std::fs`, `std::net`
- Any dependency on `planet-renderer`
