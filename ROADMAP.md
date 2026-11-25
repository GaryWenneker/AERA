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
