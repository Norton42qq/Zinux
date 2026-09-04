# Zinux Rust SDK (no_std)

Минимальный SDK для написания пользовательских программ под Zinux
(Ring 2, формат `.ZXE`) на чистом `no_std` Rust — без `alloc`, без libc.

## Состав

| Путь | Что это |
|------|---------|
| `zinux_sdk/` | библиотека SDK: syscall-обёртки, безопасный API, `#[panic_handler]` |
| `hello/` | демо-программа «Hello World from Rust!» |
| `tools/zxe-pack/` | утилита-упаковщик ELF → `.ZXE` (хостовый Rust, std) |
| `i686-zinux.json` | target spec: i686, без SSE/SSE2/MMX, no_std |
| `link.ld` | линковка кода с адреса `0x200030` |
| `.cargo/config.toml` | nightly + `-Z build-std=core` + rustflags |

## Ограничения окружения

- Вызовы ядра — только через `int 0x80` (прямые вызовы дают #GP):
  - EAX=1 → `putc` (символ в BL); EAX=2 → `exit` (код в EBX);
    EAX=3 → `api` (EBX = ID, ECX = указатель на аргументы);
- код загружается по адресу `0x200000`, исполняется с `0x200030`
  (`0x200000` + 48-байтный заголовок), стек сверху вниз от `0x2F0000`;
- argc/argv не передаются — точка входа `_start() -> !` не должна их читать;
- `ret` из `_start` запрещён — только `syscall::syscall_exit(code)`;
- готовый бинарник (заголовок + код) должен быть ≤ 64 KiB;
- вся память программы (код, данные, стек) — в диапазоне `0x200000..0x300000`.

## Сборка и запуск

```sh
# 1. Собрать образ Zinux (из корня репозитория)
make

# 2. Собрать SDK + демо + упаковать hello.zxe
make            # или: make pack

# 3. Скопировать программу в образ и запустить в QEMU
make run        # make install, затем в оболочке Zinux: HELLO
```

Нужны: `rustup nightly` (с компонентом `rust-src`),
`qemu-system-i386`, `mtools`. Смена имени/файла демо — переменные
`NAME`, `IMG` в Makefile.

## Написание программы

```rust
#![no_std]
#![no_main]

use zinux_sdk::api;
use zinux_sdk::syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    api::clear();
    api::color(api::colors::GREEN, api::colors::BLACK);
    api::println("Hello World from Rust!");
    api::print("Press any key...");
    api::waitkey();

    syscall::syscall_exit(0)
}
```

## SDK-API

```rust
// низкий уровень (inline asm, int 0x80)
syscall::syscall_putc(c: u8)
syscall::syscall_exit(code: i32) -> !
syscall::syscall_api(sys_id: u32, arg: *const ()) -> u32

// безопасный уровень (обёртки syscall_api)
api::clear()                              // CLEAR_SCREEN
api::print(&str) / println(&str) / print_char / print_int / print_hex
api::color(fg: u32, bg: u32)              // SET_COLOR, цвета 0x00RRGGBB
api::waitkey() -> u8                      // блокирующий ввод (HAS_KEY + GET_KEY)
api::has_key() -> u32 / get_key() -> u8 / clear_keys()
api::set_cursor(x, y) / api::delay(ms)
```

Паника: печатает сообщение через `putc` и завершает программу
вызовом `syscall_exit(1)`. Плавающая арифметика не используется
(SSE отключён, x87 ядро не поддерживает) — только целочисленный код.