# AERA

**One OS. Any machine.**

A Rust-first portable operating environment targeting hosted, virtualized and bare-metal execution on x86-64 and ARM64. This repository starts with a hosted prototype, not a bootable OS.

- [Specification](SPEC.md)
- [Architecture](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Performance](PERFORMANCE.md)
- [Compatibility](COMPATIBILITY.md)
- [Security](SECURITY.md)
- [Agent rules](AGENTS.md)
- [Validation](EVIDENCE.md)

## Development
Install stable Rust with rustfmt and Clippy. Run `cargo test --workspace`, `cargo run -p aera-cli -- info`, `cargo run -p aera-cli -- doctor`. License remains to be decided by the owner.

## AERA Shell (P2a prototype)
Launch with `cargo run -p aera-shell --release`. A native resizable desktop window with dock, search, independent internal panels, theme/zoom controls and read-only working-directory listing. Uses eframe/egui with wgpu, not a browser. Linux build dependencies: libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libssl-dev. A working display and GPU adapter are required; actual backend support and GUI smoke tests remain unverified. This is not a compositor or bootable OS.

## Standalone OS engineering preview
See [visual design](DESIGN.md) and [guest boot instructions](BOOT.md). The experimental guest CI builds an x86-64 Debian-backed qcow2 image, direct-boot kernel/initramfs and runs a QEMU readiness test. Artifacts are published only after the smoke test passes. Native Apple Silicon, UEFI, custom compositor and Rust kernel are not yet supported. No successful guest boot or visual verification claimed yet.

## Desktop system applications
See [DESKTOP.md](DESKTOP.md). Files supports real directory navigation, selection, file types and sizes; Disks shows actual mounted volumes and capacity and opens them in Files. Both are read-only previews. Settings links to storage and system information; the desktop app grid and dock launch these panels. No partitioning, formatting, mount changes or file mutations are implemented. Launch with `cargo run -p aera-shell --release`.
