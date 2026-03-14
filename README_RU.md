# Зинукс — Аналоговый «Линукс» #1 на Великой Руси

## ![Logo Image](assets/logo1-256.png)
[API Documentation](docs/Zinux-SDK-Documentation.md) • [Оригинальный readme](README.md)

## ——— Свиток Состояния ———

### Загрузочная кодировка исправен — 🟢 (Ядро запускает)
### Ядро исправно — 🟢 (Работает)
### Драйвер для кнопок — 🟡 (Есмь сложности)
### Своё расширение для запуска приложений (подобие .exe, .appimage) — 🟡 (SDK или же API неустойчив, аки ветер)
### Попытки запуска на настоящем ЭВМ — 🟡 (Домашняя: Работает сносно, но непостоянно. Скрижаль походная: Беда с драйвером видения)

---

## ——— Сборка и Запуск ———

### Что потребно мастеру

- NASM (не ниже 2.15)
- QEMU (не ниже 5.0)
- GNU Make

```bash
sudo pacman -S nasm, i686-elf-gcc, gcc, make, ld, qemu, mtools, dosfstools, dd
```

---

### Возведение системы

```bash
# Возвести систему
make clean && make
```

```bash
# Возвести программы
cd development\ folder/
make
```

---

### Водружение программ

**Путь первый, простой:**

```bash
cd development\ folder/
make install
```

**Путь второй, хитрый:**

```bash
mcopy -i zinux.img@@1048576 "file" ::
```

---

### Запуск во QEMU

**Путь первый:**

```bash
make run
```

**Путь второй:**

```bash
qemu-system-i386 -machine pc,usb=off -drive file=zinux.img,format=raw,index=0,media=disk -m 640K -vga std
```

---

### Запуск на настоящей ЭВМ

```bash
sudo dd if=zinux.img of=/dev/sdX bs=4M oflag=sync status=progress && sudo sync
```

#### В BIOS (Храме настроек железа):

- Запретить Secure Boot (Охрану строгую)
- Дозволить CSM (Путь совместимости)

---

### Очищение от скверны

Дабы изничтожить все следы сборки, рекеши:

```bash
make clean
```

---

*Коли всё справно — узришь, как загрузчик пробуждает Зинукс.*

---

## ——— ЭВМ, на котором испытывалась система ———

- **Машина первая:**
> Ум: I5-10400F
  Очи: AMD Radeon RX550 4GB (Рода Buffin)
  Память: 16GB DDR4 2666Mhz
  Основа: Asus PRIME b460m-k

- **Машина вторая:**
> Ум: I3-3220
  Очи: GTX650 1GB
  Память: 8GB DDR3 1600Mhz
  Основа: Asus P8H77M

- **Скрижаль походная первая** `Asus x550cc`:
> Ум: i3-2365M (UHD 4000)
  Очи: Nvidia Geforce GT720M
  Память: 4GB DDR3 1600Mhz

---

## ——— Связи и вести ———

- [Телеграм-голубятня](https://t.me/Zinux_channel)
- [Свиток жалоб на Github](https://github.com/Norton42qq/Zinux/issues)

---

## ——— История звёзд ———

<a href="https://www.star-history.com/#norton42qq/zinux&type=date&legend=top-left">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&theme=dark&legend=top-left" />
<source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&legend=top-left" />
<img alt="Свиток звёздной истории" src="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&legend=top-left" />
</picture>
</a>