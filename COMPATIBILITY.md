# Compatibility

All entries are targets, not verified support. Hosted Windows, macOS and Linux on x86-64/ARM64 require target-specific CI and real hardware smoke tests. VM guest support initially Linux-backed with virtio; validate graphics, storage, network and input. Native UEFI x86-64 then generic ARM64; Apple Silicon requires per-model qualification (including M4 Pro), no blanket support claim. RISC-V deferred.

Application strategy: portable native Rust APIs, optional WASI, separately investigate Linux ABI and Windows compatibility via isolated runtime/VM. macOS proprietary applications are not assumed portable. Host execution cannot replace the host kernel. Dual boot and bare metal depend on boot chain and drivers.

Qualification checklist: boot, display/GPU, keyboard/mouse, storage, network, audio, Bluetooth, power/sleep, security, updates, rollback. Publish exact device and software versions with evidence before labeling supported.
