# Personal WGPU Utilities

This crate contains convenience functions and wrappers for writing apps with wgpu.

## Warning

The code is extremely poorly written but needs to be public so I can depend on its github page in my other projects

## Goals

- [X] Move wgpu and winit related helpers into their own crate so different projects can have a shared base
- [ ] Test wasm support
- [ ] Support wgsl include bundling and hot reloading on native builds
- [ ] Integrate EGUI for UI