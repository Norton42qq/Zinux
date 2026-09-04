# Zinux - Analog "Linux" #1 in Russia

## ![Logo Image](assets/logo1-256.png)
[Rust SDK Documentation](rust-sdk/README.md) • [API Documentation](docs/Zinux-SDK-Documentation.md) • [Инструкция для товарищей](README_RU.md)

## --------- Check List ---------

### Working bootloader - 🟢 (Adaptive EDID resolution, boots to protected mode)

### Working kernel - 🟢 (Rust port — shell, FAT16, ZXE loader, drivers)

### Working keyboard driver - 🟢 (PS/2 keyboard + mouse)

### Your own extension for launching applications (.ZXE) - 🟢 (C SDK + Rust SDK)

### Attempts to boot on real hardware - 🟡 (Desktop works, notebook needs video driver work)

## --------- Build & Run ---------

### Prerequisites

```bash
# Arch Linux
sudo pacman -S nasm make binutils qemu-system-x86 mtools dosfstools i686-elf-gcc

# Ubuntu/Debian
sudo apt install nasm make qemu-system-x86 mtools dosfstools gcc-i686-linux-gnu
```

Rust toolchain (nightly, for kernel):
```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

---

### Build

```bash
make          # Собрать ядро + образ (zinux.img)
make run      # Собрать и запустить в QEMU
make clean    # Удалить артефакты сборки
```

### User Programs (.ZXE)

```bash
make prog          # Собрать все программы (Rust, rust-sdk/)
make prog-install  # Собрать + установить в образ
make prog-run      # Собрать + установить + запустить в QEMU
```

Программы автоматически попадают в `BIN/` и `GAMES/` на диске.

---

### Flash to USB
```bash
sudo dd if=zinux.img of=/dev/sdX bs=4M oflag=sync status=progress && sudo sync
```
BIOS settings:
- Disable Secure Boot
- Enable CSM

---

### The hardware on which the system was tested
- **PC #1**
  > CPU: I5-10400F
  > GPU: AMD Radeon RX550 4GB
  > RAM: 16GB DDR4 2666Mhz
  > MB: Asus PRIME b460m-k

- **PC #2**
  > CPU: I3-3220
  > GPU: GTX650 1GB
  > RAM: 8GB DDR3 1600Mhz
  > MB: Asus P8H77M

- **Notebook #1** `Asus x550cc`
  > CPU: i3-2365M (UHD 4000)
  > GPU: Nvidia Geforce GT720M
  > RAM: 4GB DDR3 1600Mhz

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
