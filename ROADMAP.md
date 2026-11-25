# Implementation plan

Each phase requires reviewed spec, test evidence and a checkpoint commit before marking complete. Independent review required for unsafe code, storage, security and installers.

| Phase | Deliverable | Gate |
|---|---|---|
| P0 | Docs, Cargo workspace, hosted target detection, CLI, CI | AERA-012–014 tests and cross-host builds |
| P1 | Platform adapters, config, logging, bounded task executor | platform contract tests, failure injection |
| P2 | GPU shell, compositor integration, input, accessibility | reference UI latency and accessibility tests |
| P3 | App runtime, packages, permissions, storage | isolation and crash recovery tests |
| P4 | Bootable Linux-backed VM images, virtio | unattended boot and shutdown on both ISAs |
| P5 | Native reference hardware and installer | device matrix, rollback, power/sleep tests |
| P6 | Apple Silicon research and model-specific support | boot, GPU, storage, sleep, recovery proven |
| P7 | Sync and optional compatibility layers | encrypted conflict/recovery and app matrix |
| P8 | Rust kernel feasibility prototype | benchmark and driver/ABI review; go/no-go |
| P9 | Release hardening | signed artifacts, audited updates, performance and security gates |

## P0 checklist
- [ ] Commit initial spec and plan.
- [ ] Add dependency-free Rust core and CLI tests.
- [ ] Add CI for Windows, macOS and Linux.
- [ ] Run and record test, lint, format and release results.
- [ ] Review spec and decide license.

## Change control
Every implementation PR references requirement IDs, updates affected specs and plans, and supplies test evidence. Changes to specs reopen affected checks; do not silently relax thresholds. Record alternatives and tradeoffs in ADRs. Version history is Git commits, with tags for approved baselines.

## P2 GUI work breakdown (P2a may proceed alongside P1)
- [ ] P2a: native hosted Rust shell, status bar, dock, internal windows, search, theme, zoom, read-only file listing and headless model tests (AERA-015–019 partial).
- [ ] P2b: extract reusable AERA UI components, persistent settings, async file navigation, keyboard/screen reader and contrast audits.
- [ ] P2c: window manager policies, snapping/tiling, workspaces and per-monitor DPI.
- [ ] P2d: native guest compositor and input/display adapters; external app surfaces; no host global window takeover.
- [ ] P2e: GPU backend smoke matrix, performance traces, artifact packaging, x86-64/ARM64 tests and accessibility review.
- [ ] P2f: verify all AERA-015–024 acceptance criteria with evidence before claiming GUI phase complete.

Setup: add crates/aera-shell (native binary + testable state); use eframe/egui with wgpu, default fonts, AccessKit and Linux X11/Wayland support. Dependency is justified by native event loop, GPU rendering and accessibility foundations; audit resolved transitive graph and license before release. No custom compositor dependency in P2a. CI adds Linux build packages as required. Commit spec before code and retain unchecked criteria until actual evidence.

## P4a standalone preview (parallel to P2)
- [ ] Define visual identity in DESIGN.md; implement theme and wallpaper.
- [ ] Build Debian root filesystem, Linux kernel and minimal X11 session with AERA Shell as desktop; export qcow2/kernel/initramfs.
- [ ] Boot QEMU in CI and validate serial readiness marker from actual shell window.
- [ ] Capture GUI screenshot and verify input/shutdown manually; then add ARM64 guest and UEFI boot.
Setup: disposable Linux CI builder; debootstrap, guestfish, qemu, Xorg/Openbox and Mesa software Vulkan. Existing Rust GUI dependency retained; no raw disk installation. Separate human approval pending.

## P2g — usable graphical system applications (priority before desktop polish)
- [ ] Desktop app grid, dock state and first-party registry (AERA-028/032).
- [ ] Real directory navigation, metadata, home/parent and selection (AERA-029).
- [ ] Read-only cross-platform mounted filesystem inventory and capacity (AERA-030).
- [ ] Grouped settings and storage entry point (AERA-031).
- [ ] Async service adapters and persistent preferences; safe file mutations, trash and recovery (AERA-033).
- [ ] Physical disk inventory, authorization service and mount lifecycle; defer destructive disk operations pending independent review.
- [ ] Monitor, terminal, installer, session/power actions, native compositor and standalone validation (AERA-034).
See DESKTOP.md for acceptance and dependency justification. No user disk modifications in this increment.

### P2g file actions checkpoint
- [ ] AERA-035 create folder/empty file, validation and collision protection.
- [ ] AERA-036 explicit regular-file copy, no overwrite; test error paths.
- [ ] Add list filter and navigation history; next: asynchronous I/O, safe rename/move and recoverable trash after independent review.

## P2h — desktop window lifecycle and workspaces
- [ ] Window state model and tests for minimize/restore, maximize exclusivity, workspace isolation and bounds.
- [ ] Global menu bar and dock, maximize presentation, workspace switcher and keyboard navigation.
- [ ] Separate app content per workspace, move windows between workspaces, overview and persistence.
- [ ] Validate real interaction, accessibility, geometry restore and native compositor integration.

## Glass rendering track
- [ ] G0/G1: scene texture and two-pass blur prototype.
- [ ] G2/G3/G4: WGSL optical material, tint and highlight.
- [ ] G5/G6: fluid shape interpolation, fallback and quality settings; reusable material.
- [ ] G7: ordered compositor backdrop capture and all shell surfaces.
- [ ] G8: benchmark GPU, accessibility, screenshots and regression validation. See GLASS.md.

### Glass cache and evidence checkpoint
- [ ] Verify regional luminance and theme-specific tint in real frames (AERA-048).
- [ ] Verify intermediate cache invalidation and measure pointer-only render cost (AERA-049).
- [ ] Validate dark, light and fallback screenshots from matching source commit (AERA-050).

### Desktop visual correction (priority)
- [ ] Remove desktop shader demo and redundant tiles; implement centered icon dock, top menu and window chrome.
- [ ] Files location sidebar and actual desktop/Files screenshots.
- [ ] Replace interim translucent chrome with shared compositor Glass material after backdrop capture is implemented.

### Fresh visual system
- [ ] AERA-054 implement shared vector icon registry, light-default palette, compact dock sizing and icon-labelled menus/settings. Validate live screenshots and interaction.

### Wallpaper increment
- [ ] AERA-055 built-in artwork and local PNG/JPEG/WebP, Settings selection, cover/fit, GPU texture caching; validate screenshot and decode limits. Follow with persistent preferences and native file picker.

### Window and terminal increment
- [ ] AERA-056 custom window frame and two-axis resizing, test drag and restore on real desktop.
- [ ] AERA-057 terminal command runner, bounded asynchronous output, cancel, history, cd, and icon integration. Next: PTY and ANSI terminal grid, interactive shells, tabs, per-workspace sessions.

### Terminal streaming checkpoint
- [ ] Validate bounded live stdout/stderr, UTF-8 split handling, hidden-window polling and direct child cancellation. Full PTY, process-tree termination, tabs and ANSI remain open.
