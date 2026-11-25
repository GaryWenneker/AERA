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
