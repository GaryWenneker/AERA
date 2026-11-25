# AERA product specification — 0.1 draft

Status: autonomous initial draft; separate human review pending. Requirements use stable IDs; checkboxes are editable by humans and agents. Check only with linked evidence, reopen on regression. Git history is the authoritative revision history.

## Product requirements
- [ ] AERA-001 Hosted desktop on Windows, macOS and Linux; same user-visible workflows, native host permissions.
- [ ] AERA-002 Portable core for x86-64 and ARM64, no mandatory network or cloud account.
- [ ] AERA-003 Virtual machine image boots to usable desktop via virtio; separate architecture builds.
- [ ] AERA-004 Bare-metal UEFI boot on reference x86-64 and generic ARM64 machines; drivers and recovery validated.
- [ ] AERA-005 Apple Silicon native boot only after model-specific firmware, GPU, storage, sleep and recovery support is proven; no unsupported installer.
- [ ] AERA-006 GPU-accelerated compositor and accessible keyboard-first shell, display scaling, input, clipboard, window management and settings.
- [ ] AERA-007 Sandboxed application runtime with versioned capabilities; native packages and WASI where suitable. Linux ABI and Windows compatibility are separate optional feasibility tracks, not guaranteed.
- [ ] AERA-008 Crash-consistent storage, signed atomic updates, rollback and recovery.
- [ ] AERA-009 Optional encrypted cross-device workspace synchronization with conflict detection and explicit consent; no claim of arbitrary process migration.
- [ ] AERA-010 No advertising or mandatory telemetry; local-first operation.
- [ ] AERA-011 Performance gates per PERFORMANCE.md, measured on reference hardware.

## First increment (P0)
- [ ] AERA-012 Core identifies compile-target OS and architecture; unsupported targets explicitly reported; no hardware capability claims.
- [ ] AERA-013 CLI supports info, doctor, help; unknown arguments exit 2, unsupported hosted targets exit 1 on doctor.
- [ ] AERA-014 Cross-host CI verifies formatting, lint, tests and release build; actual ARM64/x86-64 verification required before support claims.

## Acceptance
Given a supported hosted build, when doctor runs, then it reports target eligibility (not driver or system readiness) and exits 0. Given unsupported target, doctor exits 1. Given invalid arguments, CLI exits 2 with usage. Info reports compiled target. For each later feature, add positive, negative, recovery and platform tests before implementation; link test names, commit and evidence here or in EVIDENCE.md.

## Setup and decisions
Initialize empty main with documentation, then code checkpoints. Standard Rust library only for P0; Cargo, rustfmt and Clippy; no third-party dependencies. No local user working tree changes. Subsequent dependencies need ADR justification and license/security review. Public repository license choice remains open; do not assume a license. No separate spec approval obtained (autonomous run requested).

## AERA Shell — detailed GUI specification (P2)
- [ ] AERA-015 Hosted native desktop prototype: branded workspace, top status bar, dock, launchable Files, Settings and About panels, no browser runtime. Window opens at usable size and supports resize.
- [ ] AERA-016 Theme controls: light/dark palettes, consistent typography and spacing, UI zoom; persist preferences in later P1 storage integration.
- [ ] AERA-017 Search launcher: case-insensitive application filtering, keyboard focus, launch via Enter or click, empty result state.
- [ ] AERA-018 File manager: real filesystem navigation, safe read-only listing first, permission errors, later copy/move/delete with explicit confirmation and recovery. No fabricated files.
- [ ] AERA-019 Window lifecycle: open, close, focus, move, resize, minimize, maximize, snapping and tiling; hosted internal windows first, native compositor later.
- [ ] AERA-020 Multiple virtual workspaces, multi-monitor placement, DPI/Retina/4K and 120 Hz-capable presentation.
- [ ] AERA-021 Keyboard accessibility, screen reader semantics, visible focus, contrast and reduced motion. Test with actual assistive technologies.
- [ ] AERA-022 GPU backend qualification: Metal/macOS, DX12/Windows, Vulkan/Linux; record actual backend, frame pacing and fallback behavior. No continuous rendering while idle.
- [ ] AERA-023 Native compositor, input routing and external app surfaces for standalone Linux-backed guest/native mode; do not claim hosted internal windows are OS-level window management.
- [ ] AERA-024 GUI packaging, native smoke tests on x86-64/ARM64, measured startup/RSS/frame latency; no Electron or Chromium in shell.

P2a acceptance: native window launches, dock opens independent internal panels, close works, search filters app names and opens selected app, theme/zoom controls update the UI, About displays actual compiled target. Files initially exposes the user-selected host working directory in a bounded read-only listing; errors are visible. No host privilege escalation or filesystem writes. Unit tests cover search and shell state; manual GPU smoke tests remain mandatory. P2a does not establish compositor, full accessibility, 120 Hz or native hardware support.

## Standalone visual preview
- [ ] AERA-025 Consistent AERA visual tokens and procedural wallpaper per DESIGN.md, dark/light contrast, no perpetual animation.
- [ ] AERA-026 Build a disposable x86-64 Linux-backed qcow2 guest and export kernel/initramfs; boot without host OS application dependencies inside guest; start AERA graphical session as non-root.
- [ ] AERA-027 Serial boot smoke validates X server and shell window; manual screenshot and input verification required. No host disk writes or network by default. ARM64 and UEFI remain separate gates.

## Desktop system applications
Requirements AERA-028–034 and screen-level acceptance criteria are maintained in [DESKTOP.md](DESKTOP.md), a normative extension of this specification. Files and Disks are first-class system applications, not visual placeholders. Read-only inspection precedes privileged or destructive operations.

File mutation requirements AERA-035–036 extend AERA-029 and are specified in DESKTOP.md. No destructive operations authorized by this increment.

Desktop window and workspace requirements AERA-037–040 in DESKTOP.md extend AERA-019/020. Normal sessions retain a global top bar; maximized internal apps hide shell chrome and retain restore controls.

AERA-041–047 define the hybrid GPU Glass material in [GLASS.md](GLASS.md). Actual screenshot provenance is mandatory.

AERA-048–050 extend GLASS.md with regional luminance, intermediate texture caching and three-mode screenshot evidence.

AERA-053 requires a desktop-first visual correction per DESIGN.md and GLASS.md, with real screenshot evidence.
