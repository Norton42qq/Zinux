#![no_std]
#![no_main]

use zinux_sdk::api;

struct Prng {
    seed: u32,
}

impl Prng {
    fn new(seed: u32) -> Self { Prng { seed } }
    fn next(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.seed
    }
}

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    api::clear();
    api::color(api::colors::WHITE, api::colors::BLACK);

    let logo =
        "            ___\n\
         / | \\\n\
        |     |\n\
        |_____|\n\
        |     |\n\
        |     |\n\
        |     |\n\
        |     |\n\
        |     |\n\
        |     |\n\
    ___ |     | ___\n\
   /   \\       /   \\\n\
  |     |     |     |\n\
   \\___/       \\___/\n";

    api::println(logo);

    let (h, m, s) = api::get_time();
    let (d, mo, y) = api::get_date();
    let mut prng = Prng::new(h as u32 * 3600 + m as u32 * 60 + s as u32 + d as u32 + mo as u32 + y as u32);

    api::println("Введите длинну пароля:");

    let mut input_buf = [0u8; 16];
    api::read_line(&mut input_buf, 15);
    let input_str = core::str::from_utf8(&input_buf).unwrap_or("");

    let dlinna = api::str_to_int(input_str);

    if api::slen(input_str) == 0 {
        api::println("Нормально напиши еблан");
        api::waitkey();
        zinux_sdk::syscall::syscall_exit(0);
    }

    if dlinna <= 0 {
        api::println("Число ввести надо а не эту залупу");
        api::waitkey();
        zinux_sdk::syscall::syscall_exit(0);
    }

    let len = if dlinna > 255 { 255 } else { dlinna as usize };
    let charset_len = CHARSET.len();

    let mut password = [0u8; 256];
    for i in 0..len {
        let rand_val = prng.next();
        password[i] = CHARSET[(rand_val as usize) % charset_len];
    }

    api::print("Ваш пароль: ");
    api::println(core::str::from_utf8(&password[..len]).unwrap_or(""));

    api::println("");
    api::println("Нажмите любую клавишу для выхода...");
    api::waitkey();

    zinux_sdk::syscall::syscall_exit(0)
}
