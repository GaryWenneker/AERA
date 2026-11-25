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
