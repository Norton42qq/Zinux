# Зинукс — Отечественный аналог «Линукс» №1 в РСФСР

## ![Логотип](assets/logo1-256.png)
[Rust SDK — Документация](rust-sdk/README.md) • [Документация API](docs/Zinux-SDK-Documentation.md) • [Инструкция для иностранного товарища](README.md)

## --------- Список выполненных задач ---------

### Рабочий загрузочный код — 🟢 (Адаптивное разрешение EDID, загрузка в защищённый режим)

### Рабочее ядро — 🟢 (Порт на Rust — оболочка, FAT16, загрузчик ZXE, драйверы)

### Рабочий клавиатурный драйвер — 🟢 (PS/2 клавиатура + мышь)

### Собственное расширение для запуска программ (.ZXE) — 🟢 (C SDK + Rust SDK)

### Попытки запуска на реальной ЭВМ — 🟡 (Настольная ЭВМ работает, портативная требует доработки видеодрайвера)

## --------- Сборка и запуск ---------

### Необходимые инструменты

```bash
# Arch Linux
sudo pacman -S nasm make binutils qemu-system-x86 mtools dosfstools i686-elf-gcc

# Ubuntu/Debian
sudo apt install nasm make qemu-system-x86 mtools dosfstools gcc-i686-linux-gnu
```

Инструментальная цепочка Rust (ночь, для ядра):
```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

---

### Сборка

```bash
make          # Собрать ядро + образ (zinux.img)
make run      # Собрать и запустить в QEMU
make clean    # Удалить артефакты сборки
```

### Пользовательские программы (.ZXE)

```bash
make prog          # Собрать все программы (Rust, rust-sdk/)
make prog-install  # Собрать + установить в образ
make prog-run      # Собрать + установить + запустить в QEMU
```

Программы автоматически попадают в `BIN/` и `GAMES/` на диске.

---

### Прошивка на USB-накопитель
```bash
sudo dd if=zinux.img of=/dev/sdX bs=4M oflag=sync status=progress && sudo sync
```
#### Настройки БИОС
- Отключить «Безопасную загрузку» (Secure Boot)
- Включить режим совместимости КСМ (CSM)

---

### Перечень ЭВМ, на которых проводилось тестирование

- **ЭВМ №1**
  > Микропроцессор: I5-10400F
  > Видеоадаптер: AMD Radeon RX550 4ГБ
  > ОЗУ: 16ГБ DDR4 2666МГц
  > Главная плата: Asus PRIME b460m-k

- **ЭВМ №2**
  > Микропроцессор: I3-3220
  > Видеоадаптер: GTX650 1ГБ
  > ОЗУ: 8ГБ DDR3 1600МГц
  > Главная плата: Asus P8H77M

- **Портативная ЭВМ №1** `Asus x550cc`
  > Микропроцессор: i3-2365M (UHD 4000)
  > Видеоадаптер: Nvidia Geforce GT720M
  > ОЗУ: 4ГБ DDR3 1600МГц

## --------- Связь с коллективом разработчиков ---------

- [Телеграм](https://t.me/Zinux_channel)
- [Гитхаб](https://github.com/Norton42qq/Zinux/issues)

## --------- История звёзд ---------

<a href="https://www.star-history.com/#norton42qq/zinux&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&legend=top-left" />
   <img alt="График истории звёзд" src="https://api.star-history.com/svg?repos=norton42qq/zinux&type=date&legend=top-left" />
 </picture>
</a>
