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
