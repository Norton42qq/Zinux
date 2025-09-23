# Zinux - Analog "Linux" #1 in Russia

## ![Logo Image](assets/logo1-256.png)

## --------- Check List ---------

### Working bootloader - 🟧 (Infinite kernel startup)

### Working kernel - 🟥

### Working drivers for keyboard - 🟥

### Your own extension for launching applications (analog .exe, .appimage) - 🟥

### Attempts to boot on real hardware - 🟥 (UEFI = "Not booting for uefi", Legacy - "boot error")

## --------- Build & Run ---------

### Prerequisites

- NASM (>= 2.15)
- QEMU (>= 5.0)
- GNU Make

### Build

```bash
make
```

This will generate a bootable floppy image (.img) in the project root directory.

Run in QEMU

```
 qemu-system-x86_64 -fda zinux.img
```

If everything works, you should see the bootloader starting Zinux.

--------- Social ---------

- [Telegram](https://t.me/Zinux_channel)

- [Github](https://github.com/Norton42qq/Zinux/issues)

---
