#!/usr/bin/env bash
set -euo pipefail
[[ $# == 2 && $(id -u) == 0 && $(uname -m) == x86_64 ]] || { echo "Usage (root, disposable x86_64 Linux): $0 NEW_OUTPUT_DIR SHELL_BINARY" >&2; exit 2; }
binary=$(realpath "$2")
[[ -f "$binary" && -x "$binary" ]]
mkdir -- "$1"
out=$(realpath "$1")
here=$(cd -- "$(dirname -- "$0")" && pwd)
root=$(mktemp -d)
cleanup() { rm -rf -- "$root"; }
trap cleanup EXIT
debootstrap --arch=amd64 --variant=minbase bookworm "$root" https://deb.debian.org/debian
# Suppress service startup during image assembly.
printf '#!/bin/sh\nexit 101\n' > "$root/usr/sbin/policy-rc.d"
chmod 755 "$root/usr/sbin/policy-rc.d"
chroot "$root" apt-get update
chroot "$root" env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends linux-image-amd64 systemd-sysv xserver-xorg-core xserver-xorg-legacy xserver-xorg-video-vesa xserver-xorg-video-fbdev xserver-xorg-input-libinput xinit xauth openbox xdotool mesa-vulkan-drivers libvulkan1 libgl1 libegl1 libxkbcommon0 libxkbcommon-x11-0 libxcb1 libxcb-shape0 libxcb-xfixes0 libxcb-render0 libxcursor1 libxi6 libxrandr2 libfontconfig1 libwayland-client0 libssl3 fonts-dejavu-core dbus
chroot "$root" useradd --create-home --shell /bin/sh aera
install -m 755 "$binary" "$root/usr/local/bin/aera-shell"
install -m 755 "$here/aera-session.sh" "$root/usr/local/bin/aera-session"
install -m 644 "$here/aera-session.service" "$root/etc/systemd/system/aera-session.service"
printf 'allowed_users=anybody\nneeds_root_rights=yes\n' > "$root/etc/X11/Xwrapper.config"
printf 'aera\n' > "$root/etc/hostname"
printf 'LABEL=AERA_ROOT / ext4 defaults 0 1\n' > "$root/etc/fstab"
chroot "$root" systemctl enable aera-session.service
chroot "$root" systemctl mask getty@tty1.service
rm "$root/usr/sbin/policy-rc.d"
kernel=$(find "$root/boot" -maxdepth 1 -name 'vmlinuz-*' -print | sort | tail -1)
[[ -n "$kernel" ]]
version=${kernel##*/vmlinuz-}
cp "$kernel" "$out/vmlinuz"
cp "$root/boot/initrd.img-$version" "$out/initrd.img"
chroot "$root" apt-get clean
tar --numeric-owner -C "$root" -cf "$out/rootfs.tar" .
qemu-img create -f qcow2 "$out/aera.qcow2" 8G
guestfish --rw -a "$out/aera.qcow2" <<EOF
run
part-disk /dev/sda gpt
mkfs ext4 /dev/sda1
set-label /dev/sda1 AERA_ROOT
mount /dev/sda1 /
tar-in $out/rootfs.tar /
sync
EOF
rm "$out/rootfs.tar"
(cd "$out" && sha256sum aera.qcow2 vmlinuz initrd.img > SHA256SUMS)
