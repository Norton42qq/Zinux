# Зинукс - Отечественный аналог «Линукс» №1 в РСФСР

## ![Логотип](assets/logo1-256.png)
[Документация API](docs/Zinux-SDK-Documentation.md) • [Инструкция для иностранного товарища](README.md)

## --------- Список выполненных задач ---------

### Рабочий загрузочный код — 🟢 (Запускает ядро системы)

### Рабочее ядро — 🟢 (Функционирует в штатном режиме)

### Рабочие программы-помощники клавиш — 🟡 (Имеются отдельные недостатки, ведётся работа по устранению)

### Собственное расширение для запуска программ (аналог .exe, .appimage) — 🟡 (Средство Разработки (СДК) и Средство связи (АПИ) могут проявлять нестабильность. Коллектив работает над улучшением.)

### Попытки запуска на реальной ЭВМ — 🟡 (Настольная ЭВМ: работает удовлетворительно, однако не без перебоев. Портативная ЭВМ: замечены проблемы с программой-помощником видеоподсистемы.)

## --------- Сборка и запуск ---------

### Необходимые инструменты

- NASM (>= 2.15)
- QEMU (>= 5.0)
- GNU Make

```bash
sudo pacman -S nasm i686-elf-gcc gcc make ld qemu mtools dosfstools dd
```
---

### Сборка
```bash
# Сборка системы
make clean && make
```
```bash
# Сборка программ
cd development\ folder/
make
```
---

### Установка программного обеспечения
**Способ 1:**
```bash
cd development\ folder/
make install
```
**Способ 2:**
```bash
mcopy -i zinux.img@@1048576 "file" ::
```
---

### Запуск в QEMU
**Способ 1:**
```bash
make run
```

**Способ 2:**
```bash
qemu-system-i386 -machine pc,usb=off -drive file=zinux.img,format=raw,index=0,media=disk -m 640K -vga std
```
---

### Запуск на реальной ЭВМ
```bash
sudo dd if=zinux.img of=/dev/sdX bs=4M oflag=sync status=progress && sudo sync
```
#### Настройки БИОС
- Отключить функцию «Безопасная загрузка» (Secure Boot) — пережиток буржуазных систем
- Включить режим совместимости КСМ (CSM)
---

### Очистка
Для удаления всех артефактов сборки выполните:
```bash
make clean
```
---

Если все операции выполнены верно, товарищ пользователь наблюдает запуск загрузчика системы Зинукс.

---
### Перечень ЭВМ, на которых проводилось тестирование системы

- **ЭВМ №1**
  > Микропроцессор: I5-10400F
  > Видеоадаптер: AMD Radeon RX550 4ГБ (Архитектура: Buffin)
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