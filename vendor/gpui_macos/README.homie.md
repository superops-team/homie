# Homie GPUI macOS patch

This directory is copied from `zed-industries/zed` revision
`dc2a339d5d043da448a3f7ddc7c0a85c63864aad`, crate `gpui_macos`.

Homie's local patch keeps ordinary builds reproducible on machines where
Xcode's separately downloaded Metal Toolchain is not installed, by using the
precompiled `src/shaders.metallib` from the pinned GPUI revision.

`src/shaders.metallib` is the unchanged upstream shader source compiled from
that pinned revision.
