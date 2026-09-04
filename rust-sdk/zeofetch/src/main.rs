#![no_std]
#![no_main]

use zinux_sdk::api;

const LOGO: &[&str] = &[
    "              %%%%#%%%%                    ",
    "            %%*       :%%                  ",
    "           %%     :%    %#                 ",
    "           %%     .% -%:.%%%#              ",
    "           %%     %%%#......+%%%           ",
    "           #%-   %%...%%-.......%%%%       ",
    "            %%    %%......-%%=......%%%     ",
    "             %%     %%.........=%%*....%%  ",
    "             %%      %%%............+%%.%% ",
    "             *%#       %%+............%%%%  ",
    "           %%%%%%%      *%%..........%%     ",
    "        @%%-     +%       %%%%....*%%*      ",
    "       %%         %%        %%%%%%*         ",
    "     *%%          %%         %%*            ",
    "    *%%           %%          %%            ",
    "    %%   %       %%            %%           ",
    "   %%   %   -   %%             %%           ",
    "   %* %%   %.-%%#              %%%          ",
    "  %%%%%%%%%%%%    :%+#%=%=     %*.%%%%      ",
    "  %%%%%%%      :%..#..=:.%%   %%..:..%=     ",
    "     %%%%      -%........%% :%%.....%%      ",
    "      #%%%%     %%......:% %%:.....%%       ",
    "        %%%%%%%%%%%:....%%%%%....%%         ",
    "            *#%%%%%%%%%%@   #%%%%          ",
];

fn print_padded(v: u8) {
    if v < 10 { api::putchar(b'0'); }
    api::print_int(v as i32);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }
    api::clear();

    let info = api::get_sysinfo();
    let (h, m, s) = api::get_time();
    let (d, mo, yr) = api::get_date();
    let sw = api::screen_w();
    let info_x: i32 = 46;

    for (i, line) in LOGO.iter().enumerate() {
        api::set_cursor(1, 2 + i as i32);
        for &ch in line.as_bytes() {
            if ch == b'%' || ch == b'@' || ch == b'.' || ch == b':' || ch == b'-'
                || ch == b'+' || ch == b'=' || ch == b'*' || ch == b'#' {
                api::color(api::colors::WHITE, api::colors::BLACK);
            } else {
                api::color(api::colors::DARK_GREY, api::colors::BLACK);
            }
            api::putchar(ch);
        }
    }

    let logo_lines = LOGO.len() as i32;

    api::set_cursor(info_x, 2);
    api::color(api::colors::LIGHT_GREEN, api::colors::BLACK);
    api::print(api::cstr(&info.username));
    api::color(api::colors::DARK_GREY, api::colors::BLACK);
    api::print("@");
    api::color(api::colors::LIGHT_GREEN, api::colors::BLACK);
    api::print(api::cstr(&info.hostname));

    api::set_cursor(info_x, 3);
    api::color(api::colors::DARK_GREY, api::colors::BLACK);
    api::print("---------------------");

    api::set_cursor(info_x, 4);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("OS");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK); api::print(api::cstr(&info.os_name));

    api::set_cursor(info_x, 5);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Memory");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK); api::print_int(info.mem_kb as i32);
    api::print("KB");

    api::set_cursor(info_x, 6);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Arch");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK); api::print("x32");

    api::set_cursor(info_x, 7);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Shell");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK); api::print("ZSH (Zinux Shell)");

    api::set_cursor(info_x, 8);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Loader");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK); api::print("Zrub Boot Loader");

    api::set_cursor(info_x, 9);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("FS");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK); api::print("FAT16");

    api::set_cursor(info_x, 10);
    api::color(api::colors::DARK_GREY, api::colors::BLACK);
    api::print("---------------------");

    api::set_cursor(info_x, 11);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Time");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK);
    print_padded(h); api::putchar(b':'); print_padded(m); api::putchar(b':'); print_padded(s);

    api::set_cursor(info_x, 12);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Date");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK);
    print_padded(d); api::putchar(b'.'); print_padded(mo); api::putchar(b'.'); api::print_int(yr as i32);

    api::set_cursor(info_x, 13);
    api::color(api::colors::DARK_GREY, api::colors::BLACK);
    api::print("---------------------");

    let mut res_buf = [0u8; 32];
    let mut pos = 0;
    let sw_str = int_to_buf(sw as i32, &mut res_buf[pos..]);
    pos += sw_str;
    res_buf[pos] = b'x'; pos += 1;
    let sh = api::screen_h();
    let sh_str = int_to_buf(sh as i32, &mut res_buf[pos..]);
    pos += sh_str;
    let suffix = b"x256color";
    res_buf[pos..pos+suffix.len()].copy_from_slice(suffix);
    pos += suffix.len();

    api::set_cursor(info_x, 14);
    api::color(api::colors::LIGHT_CYAN, api::colors::BLACK); api::print("Resolution");
    api::color(api::colors::DARK_GREY, api::colors::BLACK); api::print(": ");
    api::color(api::colors::WHITE, api::colors::BLACK);
    for &ch in &res_buf[..pos] { api::putchar(ch); }

    let cols: [u32; 8] = [
        api::colors::RED, api::colors::GREEN, api::colors::YELLOW, api::colors::BLUE,
        api::colors::MAGENTA, api::colors::CYAN, api::colors::LIGHT_GREY, api::colors::WHITE,
    ];
    api::set_cursor(info_x, 16);
    for &c in &cols {
        api::color(c, c);
        api::print("   ");
    }
    api::color(api::colors::WHITE, api::colors::BLACK);

    api::set_cursor(0, 2 + logo_lines + 1);
    api::color(api::colors::WHITE, api::colors::BLACK);
    zinux_sdk::syscall::syscall_exit(0)
}

fn int_to_buf(mut v: i32, buf: &mut [u8]) -> usize {
    if v == 0 { buf[0] = b'0'; return 1; }
    let mut tmp = [0u8; 12];
    let mut n = 0;
    while v > 0 {
        tmp[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
    }
    for i in 0..n {
        buf[i] = tmp[n - 1 - i];
    }
    n
}
