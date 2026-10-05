# Spike D: windowed Temporal Glass candidate

This isolated winit/wgpu experiment renders a synthetic atmosphere to a texture,
samples it through an experimental glass pipeline, and draws stable opaque
foreground bars and pointer feedback. It does not render project content, decode
assets, implement a release ceremony or act as the production Viewer. Shader
fidelity, readable text and real archive UI remain future production work.

```sh
cargo test --manifest-path spikes/Cargo.toml --locked -p afterglow-spike-d
cargo run --manifest-path spikes/Cargo.toml --locked --release -p afterglow-spike-d -- 1440 900 full
```

Arguments are physical client width, height and `full`, `reduced` or `opaque`.
Set actual Windows display scaling separately. The log records actual surface
size, DPI scale, adapter/type/backend/driver, effect mode, average and p95 frame
intervals over 300 frames, and input event counts. A software/unknown adapter is
rejected as reference hardware. These are application presentation cadence
measurements, not GPU timestamp measurements or measured input latency.

Move the pointer and use `1`, `2`, `3` for effect modes, Escape to close. Resizing,
focus loss and mode changes reset the sample window; inactive/zero-sized windows
pause drawing. Slow averages automatically reduce effects or use a static opaque
surface. The explicit opaque preference survives automatic quality decisions.
Feedback and foreground remain visible in all modes. Manual responsiveness and
readability observations must accompany performance measurements.

Required matrix: each of **960×640, 1440×900, 1920×1080**, at **100%, 150%, 200%**
actual DPI, on **integrated and discrete GPUs**, with **full**, **Reduced
Transparency (`opaque`)** and **low-quality (`reduced`)** effects. All 54 cells
are pending. Warm up each cell, collect at least three 300-frame windows while
interacting, inspect automatic degradation, and record hardware/OS/driver/power/
refresh-rate details. This measurement protocol is a spike implementation choice;
SPEC's 60 FPS-or-automatic-degradation and responsive-input requirements remain.

The WGSL validator and two quality policy tests passed, and Windows x64 APIs
cross-check. No physical GPU measurement or windowed execution was possible in
this workspace. See the [evidence record](../../docs/decisions/spike-d-glass.md).
