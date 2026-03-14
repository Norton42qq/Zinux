# Zinux - Analog "Linux" #1 in Russia

## ![Logo Image](assets/logo1-256.png)
[API Documentation](docs/Zinux-SDK-Documentation.md)

## --------- Check List ---------

### Working bootloader - 🟢 (Launches the kernel)

### Working kernel - 🟢 (Working)

### Working drivers for keyboard - 🟡 (There are problems)

### Your own extension for launching applications (analog .exe, .appimage) - 🟡 (The SDK or API may be unstable.)

### Attempts to boot on real hardware - 🟡 (Desktop: Works relatively well, but not consistently Notebook: Video driver issue)

## --------- Build & Run ---------

### Prerequisites

- NASM (>= 2.15)
- QEMU (>= 5.0)
- GNU Make

```bash
sudo pacman -S nasm, i686-elf-gcc, gcc, make, ld, qemu, mtools, dosfstools, dd
```
---

### build
```bash
# Build system
make clean && make
```
```bash
#Build programs
cd development\ folder/
make
```
---

### Install programs
**Method 1:**
```bash
cd development\ folder/
make install
```
**Method 2**
```bash
mcopy -i zinux.img@@1048576 "file" ::
```
---

### Start in QEMU
**Method 1:**
```bash
make run
```

**Method 2:**
```bash
qemu-system-i386 -machine pc,usb=off -drive file=zinux.img,format=raw,index=0,media=disk -m 640K -vga std
```
---

### Start in Real PC
```bash
sudo dd if=zinux.img of=/dev/sdX bs=4M oflag=sync status=progress && sudo sync
```
#### in bios
- Disable secure boot
- Enable CSM
---

### Clean
To remove all build artifacts run:
```bash
make clean
```
---

If everything works, you should see the bootloader starting Zinux.

---
### The hardware on which the system was tested
- **PC #1**
  > CPU: I5-10400F
    GPU: AMD Radeon RX550 4GB (Architecture: Buffin)
    RAM: 16GB DDR4 2666Mhz
    Motherboard: Asus PRIME b460m-k
- **PC #2**
  > CPU: I3-3220
    GPU: GTX650 1GB
    RAM: 8GB DDR3 1600Mhz
    Motherboard: Asus P8H77M
- **Notebook #1** `Asus x550cc`
  > CPU: i3-2365M (UHD 4000)
    GPU: Nvidia Geforce GT720M
    RAM: 4GB DDR3 1600Mhz

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
