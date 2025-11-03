# Zinux - Analog "Linux" #1 in Russia

## ![Logo Image](assets/logo1-256.png)

## --------- Check List ---------

### Working bootloader - 🟧 (Infinite kernel startup)

### Working kernel - 🟥

### Working drivers for keyboard - 🟥

### Your own extension for launching applications (analog .exe, .appimage) - 🟥

### Attempts to boot on real hardware - 🟥 (UEFI = "Not booting for uefi", Legacy - "Zrub is loading, then "loading Zinux kernel" hard reset. [Video](https://t.me/Zinux_channel/49)")

## --------- Build & Run ---------

### Prerequisites

- NASM (>= 2.15)
- QEMU (>= 5.0)
- GNU Make

### Build

```bash
make
```

This will generate a bootable floppy image (zinux.img) in the build/ directory.

### Run in QEMU

```
 qemu-system-x86_64 -fda build/zinux.img
```
### Run in Real PC

```
 sudo dd if=build/zinux.img  of=/dev/sdX bs=1M
```
### Clean

To remove all build artifacts (boot.bin, zinux.img) run:

```
make clean
```

This will delete the entire build/ directory

---

If everything works, you should see the bootloader starting Zinux.

## --------- Social ---------

- [Telegram](https://t.me/Zinux_channel)

- [Github](https://github.com/Norton42qq/Zinux/issues)

## --------- Star History ---------

<a href="https://www.star-history.com/#norton42qq/zinux&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&legend=top-left" />
 </picture>
</a>
