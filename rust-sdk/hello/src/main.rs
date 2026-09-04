//! Демонстрационная программа «Hello World from Rust!» для Zinux (.ZXE).
//!
//! Сборка:
//! ```sh
//! cargo build --release -p hello
//! cargo run --release -p zxe-pack -- target/i686-zinux.json/release/hello hello.zxe hello
//! ```
//! Готовый `hello.zxe` скопировать в образ: `mcopy -i zinux.img@@1048576 hello.zxe ::/BIN/HELLO.zxe`
//! и запустить из оболочки Zinux командой `HELLO`.

#![no_std]
#![no_main]

use zinux_sdk::api;
use zinux_sdk::syscall;

const TITLE: &str = "Hello World from Rust!";

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe {
        zinux_sdk::zero_bss();
    }

    api::clear();
    api::color(api::colors::CYAN, api::colors::BLACK);
    api::println(TITLE);
    api::color(api::colors::LIGHT_GREY, api::colors::BLACK);
    api::println("Welcome to Zinux (Ring 2, no_std Rust, .ZXE)");
    api::println("");

    api::color(api::colors::LIGHT_GREEN, api::colors::BLACK);
    api::println("  syscalls : int 0x80 (EAX=1 putc / EAX=2 exit / EAX=3 api)");
    api::println("  stack    : 0x2F0000, code linked at 0x200030");
    api::println("  no_alloc : true, full no_std");

    api::color(api::colors::LIGHT_GREY, api::colors::BLACK);
    api::print("Press any key to exit...");

    let key = api::waitkey();

    api::color(api::colors::LIGHT_MAGENTA, api::colors::BLACK);
    api::print("\nExiting with code 0 (key=0x");
    api::print_hex(key as u32);
    api::println(")");

    syscall::syscall_exit(0)
}