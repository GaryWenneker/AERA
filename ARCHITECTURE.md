# Architecture

## ADR-001 — staged portability (proposed)
Hosted first: Rust application core uses host OS via narrow adapters. Virtualized phase uses a Linux kernel and virtio initially to leverage proven drivers. Native phase first uses a minimal Linux-based image; replacement Rust kernel is a separately gated research track. Do not equate a Rust shell with a Rust kernel. Evaluate Redox/Asterinas versus custom kernel with measured cost, driver maturity and ABI requirements before choosing. Shared source does not imply identical binaries.

## Boundaries
- aera-core: portable policy, configuration, capability types; eventually split no_std algorithms from hosted services.
- aera-platform: host adapters for filesystem, clock, input, display, power and networking; explicit error and capability negotiation.
- aera-shell: compositor-facing desktop, accessibility, input and window lifecycle. Investigate wgpu (Metal/DX12/Vulkan) after dependency review; native compositor integration is separate from rendering.
- aera-runtime: application isolation and versioned ABI; WASI candidate, not a full desktop API by itself.
- aera-storage: transactional metadata, bounded caches, durable writes and recovery; no bespoke filesystem until crash testing justifies it.
- aera-sync: encrypted opt-in state replication, per-device keys and conflict resolution.
- aera-boot/kernel: VM and native boot, scheduling, memory management, IPC and drivers in later tracks.

## Constraints
Avoid unbounded caches, blocking UI work, needless copies and unsafe code. Prefer safe Rust; isolate unavoidable unsafe hardware access with documented invariants, fuzzing and review. Use bounded queues/backpressure; use memory-first caching only with explicit durability guarantees. Use capability-based least privilege and host sandbox mechanisms. Build reproducibly with pinned dependencies and signed releases. No installer may modify host partitions without explicit consent, backup and tested recovery.

## Portability matrix
Hosted: Windows/macOS/Linux, target x86-64 and ARM64. VM: QEMU/virtio reference, evaluate Hyper-V and Apple Virtualization. Native: UEFI reference PC and generic ARM64, then Apple model-specific bring-up. RISC-V research only. Keep hosted platform detection separate from guest and bare-metal detection.

## ADR-002 — hosted GUI prototype
Use eframe/egui 0.31.1 with the wgpu renderer for the first hosted shell; application-specific styling and state live in aera-shell. This is a native window, not HTML/Electron, and uses existing host window and GPU APIs. The eventual AERA component library and compositor are separate boundaries; evaluate replacement of the prototype toolkit against measured latency, memory, accessibility and maintenance cost. Keep GUI dependency isolated from aera-core and aera-cli. No claims of actual backend, frame rate or hardware qualification until measured. Prefer event-driven repaint. File listing is read-only and bounded, triggered explicitly rather than per frame; move I/O off the UI thread before production. No remote access, telemetry or disk modification. Dependency version initially pinned directly; lockfile, advisory review and transitive license inventory remain release gates.
