# AERA Glass — hybrid GPU material

Approved direction: hybrid wgpu + egui, selective material across all UI, adaptive quality, opaque fallback, fluid motion. Keep text and controls sharp. No host desktop capture; sample only compositor-owned content.

- [ ] AERA-041 Scene capture into GPU texture with correct z-order and damage invalidation. First increment renders the same procedural wallpaper into a GPU texture for a glass test surface; underlying app windows are not captured yet.
- [ ] AERA-042 Multi-pass Dual Kawase blur with shared down/up pyramid, texture reuse, resize invalidation and bounded quality. First increment uses two half-resolution/full-resolution passes.
- [ ] AERA-043 WGSL material: edge refraction, superellipse mask, antialiasing, luminance tint and cursor-directed specular edge. First increment samples the wallpaper and its filtered texture, not arbitrary windows.
- [ ] AERA-044 Adaptive quality and readable text: contrast-safe foreground, opaque fallback selectable at runtime, GPU timing-driven quality tiers later.
- [ ] AERA-045 Fluid shape/size interpolation and reduced motion. First increment interpolates radius/panel width with event-driven repaint; full window morphing later.
- [ ] AERA-046 Integrate material into dock, menu, panels, menus, notifications, desktop switcher and dialogs selectively; keep content-heavy surfaces opaque enough for contrast. First increment desktop test panel and shell material settings.
- [ ] AERA-047 Screenshot provenance: actual running app, exact source commit, capture resolution, platform, renderer, theme and quality. CI artifact from live window, never a mockup represented as screenshot. Compare real light/dark and fallback captures before acceptance.

## Pipeline
Procedural wallpaper render -> downsample texture -> upsample texture -> material pass -> egui foreground. For actual compositor, replace wallpaper input with ordered scene texture excluding current element; blur cache keyed by scene damage and quality, mask in pixel coordinates, composite premultiplied alpha in linear light with explicit output color conversion. Overlapping glass requires ordered passes, not sampling the current render attachment. Native compositor migration remains open.

## Acceptance
Run shader validation, fmt, clippy, tests, release build and actual Linux display smoke. Exercise resizing, light/dark, cursor, fallback, and 640x480. No frame-rate claim without profiling. Add no new external crates for renderer (use eframe re-exported wgpu); screenshot CI uses Xvfb and Mesa software Vulkan. Software rendering evidence is not physical GPU performance. No separate approval of detailed spec beyond chosen direction.
