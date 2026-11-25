#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 ]] || exit 2
dir=$(cd -- "$1" && pwd)
log="$dir/serial.log"
: > "$log"
qemu-system-x86_64 -m 2048 -smp 2 -snapshot -nic none \
    -drive "file=$dir/aera.qcow2,if=virtio,format=qcow2" \
    -kernel "$dir/vmlinuz" -initrd "$dir/initrd.img" \
    -append 'root=LABEL=AERA_ROOT rw console=tty0 console=ttyS0 systemd.journald.forward_to_console=1' \
    -vga std -display none -serial "file:$log" -monitor none -no-reboot &
pid=$!
cleanup() { kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; }
trap cleanup EXIT
for ((i=0;i<360;i++)); do
    if grep -q 'AERA_GUI_READY' "$log"; then
        kill -0 "$pid"
        echo "Guest graphical session marker observed"
        exit 0
    fi
    if ! kill -0 "$pid"; then cat "$log"; exit 1; fi
    sleep 1
done
cat "$log"
echo "Guest GUI readiness timeout" >&2
exit 1
