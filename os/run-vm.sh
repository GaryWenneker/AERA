#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 ]] || { echo "Usage: $0 IMAGE_DIR" >&2; exit 2; }
dir=$(cd -- "$1" && pwd)
exec qemu-system-x86_64 -m 2048 -smp 2 -snapshot -nic none \
    -drive "file=$dir/aera.qcow2,if=virtio,format=qcow2" \
    -kernel "$dir/vmlinuz" -initrd "$dir/initrd.img" \
    -append 'root=LABEL=AERA_ROOT rw console=tty0 console=ttyS0 systemd.journald.forward_to_console=1' \
    -vga std -serial stdio
