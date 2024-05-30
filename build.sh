#!/usr/bin/env bash
mkdir -p esp/efi/boot
cargo b -r --target x86_64-unknown-uefi || exit 1
cp target/x86_64-unknown-uefi/release/uefileste.efi esp/efi/boot/bootx64.efi
qemu-system-x86_64 --enable-kvm -device virtio-vga-gl -cpu host -smp 4 -display gtk,gl=on \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF.fd \
    -drive format=raw,file=fat:rw:esp -m 1G -rtc base=localtime,clock=host
