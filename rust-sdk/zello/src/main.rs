#![no_std]
#![no_main]

use zinux_sdk::api;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }
    api::color(api::colors::LIGHT_GREY, api::colors::BLACK);
    api::println("Hello, Zinux!");
    api::color(api::colors::WHITE, api::colors::BLACK);
    api::waitkey();
    zinux_sdk::syscall::syscall_exit(0)
}
