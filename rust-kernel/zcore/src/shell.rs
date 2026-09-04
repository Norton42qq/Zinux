//! Перенос `kernel/shell/shell.c` + `shell.h` — командная строка Zinux.

use core::ptr;

use zlibk::mem::memset;
use zlibk::num::{atoi, itoa, strtoul};
use zlibk::str::{strcat, strchr, strcmp, strcpy, strncmp, strncpy, strlen, strtok, strtrim};

use crate::arch::cpu::system_get_info;
use crate::drivers::ata::ata_dirty_count;
use crate::drivers::ata::ata_flush;
use crate::drivers::keyboard::keyboard_wait_char;
use crate::drivers::video as v;
use crate::fat16::{
    fat16_file_exists, fat16_file_size, fat16_find_file, fat16_list_dir, fat16_mkdir,
    fat16_read_file, fat16_write_file, Fat16DirEntry,
};
use crate::zxe::{zxe_run, ZXE_NOT_FOUND};

pub const SHELL_BUFFER_SIZE: usize = 256;
pub const MAX_ARGS: usize = 16;

/// Конфигурация оболочки (`shell_config_t`).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShellConfig {
    pub username: [u8; 32],
    pub hostname: [u8; 32],
    pub prompt_style: i32,
    pub theme_color: u32,
}

pub static mut CURRENT_CONFIG: ShellConfig = ShellConfig {
    username: [0; 32],
    hostname: [0; 32],
    prompt_style: 0,
    theme_color: 0,
};

static mut CURRENT_DIR: [u8; 64] = {
    let mut d = [0u8; 64];
    d[0] = b'/';
    d
};
static mut FS_READY: i32 = 0;

/// Установка доступности ФС (`shell_set_fs_ready`).
pub unsafe fn shell_set_fs_ready(val: i32) {
    FS_READY = val;
}

/// Текущая директория (`shell_get_cwd`).
pub fn shell_get_cwd() -> *const u8 {
    unsafe { CURRENT_DIR.as_ptr() }
}

// ---------------------------------------------------------------- default

/// Дефолтный конфиг.
unsafe fn config_default() {
    strcpy(CURRENT_CONFIG.username.as_mut_ptr(), b"user\0".as_ptr());
    strcpy(CURRENT_CONFIG.hostname.as_mut_ptr(), b"zinux\0".as_ptr());
    CURRENT_CONFIG.prompt_style = 0;
    CURRENT_CONFIG.theme_color = v::COLOR_LIGHT_GREEN;
}

unsafe fn parse_kv(line: *mut u8) {
    let mut eq = line;
    while *eq != 0 && *eq != b'=' {
        eq = eq.add(1);
    }
    if *eq == 0 {
        return;
    }
    let val = eq.add(1);
    let mut vlen = strlen(val);
    while vlen > 0 && (*val.add(vlen - 1) == b'\r' || *val.add(vlen - 1) == b'\n') {
        vlen -= 1;
        *val.add(vlen) = 0;
    }
    if strncmp(line, b"user=\0".as_ptr(), 5) == 0 {
        strncpy(CURRENT_CONFIG.username.as_mut_ptr(), val, 31);
        CURRENT_CONFIG.username[31] = 0;
    } else if strncmp(line, b"host=\0".as_ptr(), 5) == 0 {
        strncpy(CURRENT_CONFIG.hostname.as_mut_ptr(), val, 31);
        CURRENT_CONFIG.hostname[31] = 0;
    } else if strncmp(line, b"style=\0".as_ptr(), 6) == 0 {
        CURRENT_CONFIG.prompt_style = atoi(val);
    } else if strncmp(line, b"color=\0".as_ptr(), 6) == 0 {
        CURRENT_CONFIG.theme_color = strtoul(val, ptr::null_mut(), 16);
    }
}

/// Загрузка конфигурации из CONF/CONFIG (`shell_load_config`).
pub unsafe fn shell_load_config() {
    config_default();

    let mut cbuf = [0u8; SHELL_BUFFER_SIZE];
    let n = fat16_read_file(b"CONF/CONFIG\0".as_ptr(), cbuf.as_mut_ptr(), 255);
    if n <= 0 {
        return;
    }
    cbuf[n as usize] = 0;
    let mut line = strtok(cbuf.as_mut_ptr(), b"\n\0".as_ptr());
    while !line.is_null() {
        parse_kv(line);
        line = strtok(ptr::null_mut(), b"\n\0".as_ptr());
    }
}

/// Сохранение конфигурации (`shell_save_config`).
pub unsafe fn shell_save_config() {
    let mut cbuf = [0u8; SHELL_BUFFER_SIZE];
    let mut n: usize = 0;

    strcpy(cbuf.as_mut_ptr().add(n), b"user=\0".as_ptr());
    n += 5;
    strcpy(cbuf.as_mut_ptr().add(n), CURRENT_CONFIG.username.as_ptr());
    n += strlen(CURRENT_CONFIG.username.as_ptr());
    cbuf[n] = b'\n';
    n += 1;

    strcpy(cbuf.as_mut_ptr().add(n), b"host=\0".as_ptr());
    n += 5;
    strcpy(cbuf.as_mut_ptr().add(n), CURRENT_CONFIG.hostname.as_ptr());
    n += strlen(CURRENT_CONFIG.hostname.as_ptr());
    cbuf[n] = b'\n';
    n += 1;

    strcpy(cbuf.as_mut_ptr().add(n), b"style=\0".as_ptr());
    n += 6;
    itoa(CURRENT_CONFIG.prompt_style, cbuf.as_mut_ptr().add(n), 10);
    n += strlen(cbuf.as_mut_ptr().add(n));
    cbuf[n] = b'\n';
    n += 1;

    strcpy(cbuf.as_mut_ptr().add(n), b"color=\0".as_ptr());
    n += 6;
    itoa((CURRENT_CONFIG.theme_color & 0xFF_FFFF) as i32, cbuf.as_mut_ptr().add(n), 16);
    n += strlen(cbuf.as_mut_ptr().add(n));
    cbuf[n] = b'\n';
    n += 1;
    cbuf[n] = 0;

    if fat16_write_file(b"CONF/CONFIG\0".as_ptr(), cbuf.as_ptr(), n as u32) > 0 {
        v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
        v::video_print(b"  [Config saved to CONF/CONFIG]\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
        ata_flush();
    } else {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b"  [Failed to save config]\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    }
}

// ---------------------------------------------------------------- ввод

static mut INPUT_BUFFER: [u8; SHELL_BUFFER_SIZE] = [0; SHELL_BUFFER_SIZE];
static mut INPUT_POS: i32 = 0;

/// Структура команды.
struct Command {
    name: *const u8,
    description: *const u8,
    handler: unsafe extern "C" fn(i32, *mut *mut u8),
}

unsafe impl Sync for Command {}

// Таблица команд.
static COMMANDS: [Command; 20] = [
    Command {
        name: b"help\0".as_ptr(),
        description: b"Show available commands\0".as_ptr(),
        handler: cmd_help,
    },
    Command {
        name: b"clear\0".as_ptr(),
        description: b"Clear the screen\0".as_ptr(),
        handler: cmd_clear,
    },
    Command {
        name: b"info\0".as_ptr(),
        description: b"Show system information\0".as_ptr(),
        handler: cmd_info,
    },
    Command {
        name: b"echo\0".as_ptr(),
        description: b"Print text to screen\0".as_ptr(),
        handler: cmd_echo,
    },
    Command {
        name: b"reboot\0".as_ptr(),
        description: b"Reboot the computer\0".as_ptr(),
        handler: cmd_reboot,
    },
    Command {
        name: b"halt\0".as_ptr(),
        description: b"Halt the system\0".as_ptr(),
        handler: cmd_halt,
    },
    Command {
        name: b"time\0".as_ptr(),
        description: b"Show current time\0".as_ptr(),
        handler: cmd_time,
    },
    Command {
        name: b"date\0".as_ptr(),
        description: b"Show current date\0".as_ptr(),
        handler: cmd_date,
    },
    Command {
        name: b"version\0".as_ptr(),
        description: b"Show kernel version\0".as_ptr(),
        handler: cmd_version,
    },
    Command {
        name: b"mem\0".as_ptr(),
        description: b"Show memory information\0".as_ptr(),
        handler: cmd_mem,
    },
    Command {
        name: b"cpu\0".as_ptr(),
        description: b"Show CPU information\0".as_ptr(),
        handler: cmd_cpu,
    },
    Command {
        name: b"set\0".as_ptr(),
        description: b"set user|host|style|color\0".as_ptr(),
        handler: cmd_set,
    },
    Command {
        name: b"cd\0".as_ptr(),
        description: b"cd <dir> - change directory\0".as_ptr(),
        handler: cmd_cd,
    },
    Command {
        name: b"ls\0".as_ptr(),
        description: b"List files on disk\0".as_ptr(),
        handler: cmd_ls,
    },
    Command {
        name: b"mkdir\0".as_ptr(),
        description: b"mkdir <dir>\0".as_ptr(),
        handler: cmd_mkdir,
    },
    Command {
        name: b"cat\0".as_ptr(),
        description: b"Display file contents\0".as_ptr(),
        handler: cmd_cat,
    },
    Command {
        name: b"sync\0".as_ptr(),
        description: b"Flush disk writes to media\0".as_ptr(),
        handler: cmd_sync,
    },
    Command {
        name: b"rustest\0".as_ptr(),
        description: b"Displaying all Russian letters\0".as_ptr(),
        handler: cmd_rustest,
    },
    Command {
        name: b"die\0".as_ptr(),
        description: b"Force an exception (debug)\0".as_ptr(),
        handler: cmd_die,
    },
    Command {
        name: ptr::null(),
        description: ptr::null(),
        handler: cmd_help,
    },
];

// ---------------------------------------------------------------- приветствие

/// Приветствие (`shell_print_welcome`).
pub unsafe fn shell_print_welcome() {
    v::video_clear(v::COLOR_BLACK);
    v::video_set_color(v::COLOR_LIGHT_CYAN, v::COLOR_BLACK);
    v::video_print(b"========================================\n\0".as_ptr());
    v::video_print(b"        \xD0\x94\xD0\xBE\xD0\xB1\xD1\x80\xD0\xBE \xD0\xBF\xD0\xBE\xD0\xB6\xD0\xB0\xD0\xBB\xD0\xBE\xD0\xB2\xD0\xB0\xD1\x82\xD1\x8C \xD0\xB2 Zinux\n\0".as_ptr());
    v::video_print(b"   \xD0\x9D\xD0\x9E\xD0\x92\xD0\x90\xD0\xAF \xD0\xA0\xD0\xA3\xD0\xA1\xD0\xA1\xD0\x9A\xD0\x90\xD0\xAF \xD0\x9E\xD0\x9F\xD0\x95\xD0\xA0\xD0\x90\xD0\xA6\xD0\x98\xD0\x9E\xD0\x9D\xD0\x9D\xD0\x90\xD0\xAF \xD0\xA1\xD0\x98\xD0\xA1\xD0\xA2\xD0\x95\xD0\x9C\xD0\x90\n\0".as_ptr());
    v::video_print(b"========================================\n\n\0".as_ptr());

    v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
    v::video_print(b"\xD0\x9D\xD0\xB0\xD0\xBF\xD0\xB8\xD1\x88\xD0\xB8\xD1\x82\xD0\xB5 'help' \xD0\xB4\xD0\xBB\xD1\x8F \xD0\xB2\xD1\x8B\xD0\xB2\xD0\xBE\xD0\xB4\xD0\xB0 \xD1\x81\xD0\xBF\xD0\xB8\xD1\x81\xD0\xBA\xD0\xB0 \xD0\xBA\xD0\xBE\xD0\xBC\xD0\xB0\xD0\xBD\xD0\xB4.\n\n\0".as_ptr());

    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

/// Стили промпта (`shell_prompt`).
pub unsafe fn shell_prompt() {
    v::video_set_color(CURRENT_CONFIG.theme_color, v::COLOR_BLACK);

    match CURRENT_CONFIG.prompt_style {
        1 => {
            // user@host >
            v::video_print(CURRENT_CONFIG.username.as_ptr());
            v::video_print_char(b'@');
            v::video_print(CURRENT_CONFIG.hostname.as_ptr());
            v::video_print(b" > \0".as_ptr());
        }
        2 => {
            // user#
            v::video_print(CURRENT_CONFIG.username.as_ptr());
            v::video_print(b"# \0".as_ptr());
        }
        3 => {
            v::video_print(CURRENT_CONFIG.username.as_ptr());
            v::video_print(b"\xC8\0".as_ptr());
        }
        _ => {
            // Стандарт: user@host:~$
            v::video_print(CURRENT_CONFIG.username.as_ptr());
            v::video_print_char(b'@');
            v::video_print(CURRENT_CONFIG.hostname.as_ptr());
            v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
            v::video_print(b":~$ \0".as_ptr());
        }
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

// ---------------------------------------------------------------- парсинг

/// Парс оболочки (`parse_command`).
unsafe fn parse_command(cl: *mut u8, argv: *mut *mut u8) -> i32 {
    let mut argc = 0;
    let mut t = strtok(cl, b" \t\0".as_ptr());
    while !t.is_null() && argc < (MAX_ARGS as i32) - 1 {
        *argv.add(argc as usize) = t;
        argc += 1;
        t = strtok(ptr::null_mut(), b" \t\0".as_ptr());
    }
    *argv.add(argc as usize) = ptr::null_mut();
    argc
}

unsafe fn read_line() {
    INPUT_POS = 0;
    memset(INPUT_BUFFER.as_mut_ptr(), 0, SHELL_BUFFER_SIZE);
    loop {
        let c = keyboard_wait_char();
        if c == b'\n' {
            v::video_print_char(b'\n');
            INPUT_BUFFER[INPUT_POS as usize] = 0;
            return;
        }
        if c == b'\x08' {
            if INPUT_POS > 0 {
                INPUT_POS -= 1;
                v::video_backspace();
            }
            continue;
        }
        if INPUT_POS < (SHELL_BUFFER_SIZE as i32) - 1 {
            INPUT_BUFFER[INPUT_POS as usize] = c;
            INPUT_POS += 1;
            v::video_print_char(c);
        }
    }
}

/// Попытка запустить .ZXE (`try_run`).
unsafe fn try_run(name: *const u8, argc: i32, argv: *mut *mut u8) -> i32 {
    let mut p = [0u8; 64];
    let r: i32;

    if !strchr(name, b'/' as i32).is_null() {
        return zxe_run(name, argc, argv);
    }
    if strcmp(CURRENT_DIR.as_ptr(), b"/\0".as_ptr()) == 0 {
        r = zxe_run(name, argc, argv);
    } else {
        strcpy(p.as_mut_ptr(), CURRENT_DIR.as_ptr().add(1));
        strcat(p.as_mut_ptr(), b"/\0".as_ptr());
        strcat(p.as_mut_ptr(), name);
        r = zxe_run(p.as_ptr(), argc, argv);
    }
    if r != ZXE_NOT_FOUND {
        return r;
    }

    strcpy(p.as_mut_ptr(), b"BIN/\0".as_ptr());
    strcat(p.as_mut_ptr(), name);
    let r = zxe_run(p.as_ptr(), argc, argv);
    if r != ZXE_NOT_FOUND {
        return r;
    }

    ZXE_NOT_FOUND
}

/// Выполнение команды (`shell_execute`).
pub unsafe fn shell_execute(cmdline: *const u8) {
    let mut buffer = [0u8; SHELL_BUFFER_SIZE];
    let mut argv: [*mut u8; MAX_ARGS] = [ptr::null_mut(); MAX_ARGS];

    strncpy(buffer.as_mut_ptr(), cmdline, SHELL_BUFFER_SIZE - 1);
    buffer[SHELL_BUFFER_SIZE - 1] = 0;
    strtrim(buffer.as_mut_ptr());

    if buffer[0] == 0 {
        return;
    }

    let argc = parse_command(buffer.as_mut_ptr(), argv.as_mut_ptr());
    if argc == 0 {
        return;
    }

    // Поиск встроенной команды.
    for i in 0..COMMANDS.len() {
        let name = COMMANDS[i].name;
        if name.is_null() {
            break;
        }
        if strcmp(argv[0], name) == 0 {
            (COMMANDS[i].handler)(argc, argv.as_mut_ptr());
            return;
        }
    }

    // Попытка запустить как .zxe.
    let result = try_run(argv[0], argc - 1, argv.as_mut_ptr().add(1));
    if result != -1 && result != -2 && result != -3 && result != -4 {
        return;
    }
    if result == -1 {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b"Read error: \0".as_ptr());
        v::video_print(argv[0]);
        v::video_print(b"\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
        return;
    }

    // Команда не найдена.
    v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
    v::video_print(b"Command not found: \0".as_ptr());
    v::video_print(argv[0]);
    v::video_print(b"\n\0".as_ptr());
    v::video_print(b"Type 'help' or 'ls' for available commands.\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

// ---------------------------------------------------------------- инициализация

/// Инициализация оболочки (`shell_init`).
pub unsafe fn shell_init() {
    INPUT_POS = 0;
    memset(INPUT_BUFFER.as_mut_ptr(), 0, SHELL_BUFFER_SIZE);
    shell_load_config();
}

/// Главный цикл оболочки (`shell_run`).
pub unsafe fn shell_run() {
    shell_print_welcome();

    loop {
        shell_prompt();
        read_line();
        shell_execute(INPUT_BUFFER.as_ptr());
    }
}

// ---------------------------------------------------------------- команды

unsafe extern "C" fn cmd_cd(argc: i32, argv: *mut *mut u8) {
    // Без аргументов показывает текущий путь.
    if argc < 2 {
        v::video_print(CURRENT_DIR.as_ptr());
        v::video_print(b"\n\0".as_ptr());
        return;
    }

    let target = *argv.add(1) as *const u8;

    if strcmp(target, b"/\0".as_ptr()) == 0 {
        unsafe {
            strcpy(CURRENT_DIR.as_mut_ptr(), b"/\0".as_ptr());
        }
        return;
    }
    if strcmp(target, b"..\0".as_ptr()) == 0 {
        if strcmp(CURRENT_DIR.as_ptr(), b"/\0".as_ptr()) != 0 {
            let mut len = strlen(CURRENT_DIR.as_ptr());
            while len > 0 && CURRENT_DIR[len - 1] != b'/' {
                len -= 1;
            }
            if len > 1 {
                len -= 1;
            }
            CURRENT_DIR[len] = 0;
            if len == 0 {
                strcpy(CURRENT_DIR.as_mut_ptr(), b"/\0".as_ptr());
            }
        }
        return;
    }

    let mut newpath = [0u8; 64];
    let mut dirpart = [0u8; 32];

    if *target == b'/' {
        strcpy(newpath.as_mut_ptr(), target);
        strcpy(dirpart.as_mut_ptr(), target.add(1));
    } else {
        if strcmp(CURRENT_DIR.as_ptr(), b"/\0".as_ptr()) == 0 {
            newpath[0] = b'/';
            newpath[1] = 0;
            strcat(newpath.as_mut_ptr(), target);
            strcpy(dirpart.as_mut_ptr(), target);
        } else {
            v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
            v::video_print(b"cd: cannot go deeper than one level\n\0".as_ptr());
            v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
            return;
        }
    }

    let mut plen = strlen(dirpart.as_ptr());
    if plen > 1 && dirpart[plen - 1] == b'/' {
        plen -= 1;
        dirpart[plen] = 0;
    }

    if dirpart[0] == 0 {
        strcpy(CURRENT_DIR.as_mut_ptr(), b"/\0".as_ptr());
        return;
    }

    let mut entry = Fat16DirEntry {
        name: [0; 8],
        ext: [0; 3],
        attributes: 0,
        reserved: 0,
        create_time_ms: 0,
        create_time: 0,
        create_date: 0,
        access_date: 0,
        cluster_high: 0,
        modify_time: 0,
        modify_date: 0,
        cluster_low: 0,
        file_size: 0,
    };
    let mut found = 0;

    if fat16_find_file(dirpart.as_ptr(), &mut entry) == 0 && (entry.attributes & 0x10) != 0 {
        found = 1;
    }

    if found != 0 {
        strcpy(CURRENT_DIR.as_mut_ptr(), newpath.as_ptr());
        let l = strlen(CURRENT_DIR.as_ptr());
        if l > 1 && CURRENT_DIR[l - 1] == b'/' {
            CURRENT_DIR[l - 1] = 0;
        }
    } else {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b"cd: \0".as_ptr());
        v::video_print(target);
        v::video_print(b": No such directory\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    }
}

unsafe extern "C" fn cmd_set(argc: i32, argv: *mut *mut u8) {
    if argc < 3 {
        v::video_print(b"Usage: set <user|host|style|color> <value>\n\0".as_ptr());
        return;
    }
    if strcmp(*argv.add(1), b"user\0".as_ptr()) == 0 {
        strncpy(CURRENT_CONFIG.username.as_mut_ptr(), *argv.add(2), 31);
        CURRENT_CONFIG.username[31] = 0;
        v::video_print(b"Username: \0".as_ptr());
        v::video_print(CURRENT_CONFIG.username.as_ptr());
        v::video_print(b"\n\0".as_ptr());
    } else if strcmp(*argv.add(1), b"host\0".as_ptr()) == 0 {
        strncpy(CURRENT_CONFIG.hostname.as_mut_ptr(), *argv.add(2), 31);
        CURRENT_CONFIG.hostname[31] = 0;
        v::video_print(b"Hostname: \0".as_ptr());
        v::video_print(CURRENT_CONFIG.hostname.as_ptr());
        v::video_print(b"\n\0".as_ptr());
    } else if strcmp(*argv.add(1), b"style\0".as_ptr()) == 0 {
        CURRENT_CONFIG.prompt_style = atoi(*argv.add(2));
        v::video_print(b"Style: \0".as_ptr());
        v::video_put_dec(CURRENT_CONFIG.prompt_style as u32);
        v::video_print(b"\n\0".as_ptr());
    } else if strcmp(*argv.add(1), b"color\0".as_ptr()) == 0 {
        CURRENT_CONFIG.theme_color = strtoul(*argv.add(2), ptr::null_mut(), 16);
        v::video_print(b"Color: #\0".as_ptr());
        v::video_put_hex(CURRENT_CONFIG.theme_color);
        v::video_print(b"\n\0".as_ptr());
    } else {
        v::video_print(b"Unknown: \0".as_ptr());
        v::video_print(*argv.add(1));
        v::video_print(b"\n\0".as_ptr());
        return;
    }
    shell_save_config();
}

unsafe extern "C" fn cmd_mkdir(argc: i32, argv: *mut *mut u8) {
    if FS_READY == 0 {
        v::video_print(b"No filesystem mounted.\n\0".as_ptr());
        return;
    }
    if argc < 2 {
        v::video_print(b"Usage: mkdir <dir>\n\0".as_ptr());
        return;
    }
    if fat16_mkdir(*argv.add(1)) == 0 {
        v::video_print(b"Created: /\0".as_ptr());
        v::video_print(*argv.add(1));
        v::video_print(b"\n\0".as_ptr());
        ata_flush();
    } else {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b"Failed\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    }
}

unsafe extern "C" fn cmd_die(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);
    v::video_print(b"\nForcing #BP via int3...\n\0".as_ptr());
    core::arch::asm!("int3");
}

unsafe extern "C" fn cmd_help(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b"\n=== Available Commands ===\n\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);

    for i in 0..COMMANDS.len() {
        let name = COMMANDS[i].name;
        if name.is_null() {
            break;
        }
        v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
        v::video_print(b"  \0".as_ptr());
        v::video_print(name);

        let len = strlen(name);
        let mut j = len;
        while j < 12 {
            v::video_print_char(b' ');
            j += 1;
        }

        v::video_set_color(v::COLOR_LIGHT_GREY, v::COLOR_BLACK);
        v::video_print(b"- \0".as_ptr());
        v::video_print(COMMANDS[i].description);
        v::video_print(b"\n\0".as_ptr());
    }

    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print(b"\n\0".as_ptr());
}

unsafe extern "C" fn cmd_clear(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);
    v::video_clear(v::COLOR_BLACK);
}

unsafe extern "C" fn cmd_info(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    let si = system_get_info();

    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b"\n=== System Information ===\n\n\0".as_ptr());

    v::video_set_color(v::COLOR_LIGHT_CYAN, v::COLOR_BLACK);
    v::video_print(b"CPU:\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print(b"  Vendor:   \0".as_ptr());
    v::video_print((*si).cpu_vendor.as_ptr() as *const u8);
    v::video_print(b"\n\n\0".as_ptr());

    v::video_set_color(v::COLOR_LIGHT_CYAN, v::COLOR_BLACK);
    v::video_print(b"Memory:\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print(b"  Lower:    \0".as_ptr());
    v::video_put_dec((*si).mem_lower);
    v::video_print(b" KB\n\0".as_ptr());
    v::video_print(b"  Upper:    \0".as_ptr());
    v::video_put_dec((*si).mem_upper);
    v::video_print(b" KB\n\n\0".as_ptr());
}

unsafe extern "C" fn cmd_echo(argc: i32, argv: *mut *mut u8) {
    let mut i = 1;
    while i < argc {
        v::video_print(*argv.add(i as usize));
        if i < argc - 1 {
            v::video_print_char(b' ');
        }
        i += 1;
    }
    v::video_print_char(b'\n');
}

unsafe extern "C" fn cmd_reboot(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b"\nSyncing disk...\0".as_ptr());
    let dirty = ata_dirty_count();
    if dirty > 0 {
        let r = ata_flush();
        if r == 0 {
            v::video_print(b" OK\n\0".as_ptr());
        } else {
            v::video_print(b" ERRORS\n\0".as_ptr());
        }
    } else {
        v::video_print(b" nothing to sync\n\0".as_ptr());
    }
    v::video_print(b"Rebooting system...\n\0".as_ptr());

    for _ in 0..10_000_000 {
        // обычно пауза перед отключением прерываний
    }
    crate::arch::cpu::system_reboot();
}

unsafe extern "C" fn cmd_halt(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);
    let dirty = ata_dirty_count();
    if dirty > 0 {
        v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
        v::video_print(b"\nSyncing disk...\0".as_ptr());
        let r = ata_flush();
        if r == 0 {
            v::video_print(b" OK\n\0".as_ptr());
        } else {
            v::video_print(b" ERRORS\n\0".as_ptr());
        }
    }
    v::video_clear(v::COLOR_BLACK);
    v::video_set_color(v::COLOR_BROWN, v::COLOR_BLACK);
    v::video_puts(
        225,
        250,
        b"It's now safe to turn off your computer\0".as_ptr(),
        v::COLOR_BROWN,
        v::COLOR_BLACK,
    );
    crate::arch::cpu::system_halt();
}

/// Чтение CMOS-регистра.
unsafe fn cmos_read(reg: u8) -> u8 {
    crate::arch::io::outb(0x70, reg);
    crate::arch::io::inb(0x71)
}

fn bcd_to_bin(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

unsafe extern "C" fn cmd_time(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    let second = bcd_to_bin(cmos_read(0x00));
    let minute = bcd_to_bin(cmos_read(0x02));
    let hour = bcd_to_bin(cmos_read(0x04));

    v::video_print(b"Current time: \0".as_ptr());
    if hour < 10 {
        v::video_print_char(b'0');
    }
    v::video_put_dec(hour as u32);
    v::video_print_char(b':');
    if minute < 10 {
        v::video_print_char(b'0');
    }
    v::video_put_dec(minute as u32);
    v::video_print_char(b':');
    if second < 10 {
        v::video_print_char(b'0');
    }
    v::video_put_dec(second as u32);
    v::video_print(b"\n\0".as_ptr());
}

unsafe extern "C" fn cmd_date(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    let day = bcd_to_bin(cmos_read(0x07));
    let month = bcd_to_bin(cmos_read(0x08));
    let year = bcd_to_bin(cmos_read(0x09));
    let mut century = bcd_to_bin(cmos_read(0x32));

    if century == 0 {
        century = 20;
    }

    v::video_print(b"Current date: \0".as_ptr());
    if day < 10 {
        v::video_print_char(b'0');
    }
    v::video_put_dec(day as u32);
    v::video_print_char(b'/');
    if month < 10 {
        v::video_print_char(b'0');
    }
    v::video_put_dec(month as u32);
    v::video_print_char(b'/');
    v::video_put_dec(century as u32);
    if year < 10 {
        v::video_print_char(b'0');
    }
    v::video_put_dec(year as u32);
    v::video_print(b"\n\0".as_ptr());
}

unsafe extern "C" fn cmd_version(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    v::video_print(b"Zinux version 0.2 Beta-branch\n\0".as_ptr());
    v::video_print(b"update: support mouse (not), update sdk (GUI), new video-driver\0".as_ptr());
}

unsafe extern "C" fn cmd_mem(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    let si = system_get_info();

    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b"\n=== Memory Information ===\n\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);

    v::video_print(b"Conventional memory: \0".as_ptr());
    v::video_put_dec((*si).mem_lower);
    v::video_print(b" KB\n\0".as_ptr());

    v::video_print(b"Extended memory:     \0".as_ptr());
    v::video_put_dec((*si).mem_upper);
    v::video_print(b" KB\n\0".as_ptr());

    let total = ((*si).mem_lower + (*si).mem_upper + 1024) / 1024;
    v::video_print(b"Total memory:        ~\0".as_ptr());
    v::video_put_dec(total);
    v::video_print(b" MB\n\n\0".as_ptr());
}

unsafe extern "C" fn cmd_cpu(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);

    let si = system_get_info();

    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b"\n=== CPU Information ===\n\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);

    v::video_print(b"Vendor:   \0".as_ptr());
    v::video_print((*si).cpu_vendor.as_ptr() as *const u8);
    v::video_print(b"\n\0".as_ptr());

    v::video_print(b"Features: \0".as_ptr());
    let feat = (*si).cpu_features;
    if feat & (1 << 0) != 0 {
        v::video_print(b"FPU \0".as_ptr());
    }
    if feat & (1 << 4) != 0 {
        v::video_print(b"TSC \0".as_ptr());
    }
    if feat & (1 << 5) != 0 {
        v::video_print(b"MSR \0".as_ptr());
    }
    if feat & (1 << 23) != 0 {
        v::video_print(b"MMX \0".as_ptr());
    }
    if feat & (1 << 25) != 0 {
        v::video_print(b"SSE \0".as_ptr());
    }
    if feat & (1 << 26) != 0 {
        v::video_print(b"SSE2 \0".as_ptr());
    }
    v::video_print(b"\n\n\0".as_ptr());
}

unsafe extern "C" fn cmd_ls(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);
    if FS_READY == 0 {
        v::video_print(b"No filesystem mounted.\n\0".as_ptr());
        return;
    }
    fat16_list_dir(CURRENT_DIR.as_ptr());
}

static mut FILE_BUFFER: [u8; 64 * 1024] = [0; 64 * 1024];

unsafe extern "C" fn cmd_cat(argc: i32, argv: *mut *mut u8) {
    if FS_READY == 0 {
        v::video_print(b"No filesystem mounted.\n\0".as_ptr());
        return;
    }
    if argc < 2 {
        v::video_print(b"Usage: cat <filename>\n\0".as_ptr());
        return;
    }
    let mut filepath = [0u8; 64];
    if **argv.add(1) == b'/' || strcmp(CURRENT_DIR.as_ptr(), b"/\0".as_ptr()) == 0 {
        if **argv.add(1) == b'/' {
            strncpy(filepath.as_mut_ptr(), (*argv.add(1)).add(1), 63);
        } else {
            strncpy(filepath.as_mut_ptr(), *argv.add(1), 63);
        }
    } else {
        strncpy(filepath.as_mut_ptr(), CURRENT_DIR.as_ptr().add(1), 31);
        strcat(filepath.as_mut_ptr(), b"/\0".as_ptr());
        strcat(filepath.as_mut_ptr(), *argv.add(1));
    }
    filepath[63] = 0;

    if fat16_file_exists(filepath.as_ptr()) == 0 {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b"File not found: \0".as_ptr());
        v::video_print(filepath.as_ptr());
        v::video_print(b"\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
        return;
    }
    let mut size = fat16_file_size(filepath.as_ptr());
    if size == 0 {
        v::video_print(b"File is empty\n\0".as_ptr());
        return;
    }
    if size > 64 * 1024 {
        size = 64 * 1024;
        v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
        v::video_print(b"Warning: showing first 64KB only\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    }

    let bytes_read = fat16_read_file(filepath.as_ptr(), FILE_BUFFER.as_mut_ptr(), size);
    if bytes_read < 0 {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b"Error reading file\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
        return;
    }

    v::video_print(b"\n\0".as_ptr());
    let mut i = 0;
    while i < bytes_read {
        let c = FILE_BUFFER[i as usize];
        if c >= 32 || c == b'\n' || c == b'\r' || c == b'\t' {
            v::video_print_char(c);
        }
        i += 1;
    }
    v::video_print(b"\n\0".as_ptr());
}

unsafe extern "C" fn cmd_sync(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);
    let dirty = ata_dirty_count();
    if dirty == 0 {
        v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
        v::video_print(b"Disk is clean, nothing to sync\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
        return;
    }
    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b"Syncing \0".as_ptr());
    v::video_put_dec(dirty as u32);
    v::video_print(b" sector(s) to disk...\0".as_ptr());
    let r = ata_flush();
    if r == 0 {
        v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
        v::video_print(b" OK\n\0".as_ptr());
    } else {
        v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
        v::video_print(b" FAILED (\0".as_ptr());
        v::video_put_dec((-r) as u32);
        v::video_print(b" errors)\n\0".as_ptr());
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

unsafe extern "C" fn cmd_rustest(argc: i32, argv: *mut *mut u8) {
    let _ = (argc, argv);
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print(
        b"\x80 \x81 \x82 \x83 \x84 \x85 \x86 \x87 \x88 \x89 \x8A \x8B \x8C \x8D \x8E \x8F \x90 \x91 \x92 \x93 \x94 \x95 \x96 \x97 \x98 \x99 \x9A \x9B \x9C \x9D \x9E \x9F\n\xA0 \xA1 \xA2 \xA3 \xA4 \xA5 \xA6 \xA7 \xA8 \xA9 \xAA \xAB \xAC \xAD \xAE \xAF \xE0 \xE1 \xE2 \xE3 \xE4 \xE5 \xE6 \xE7 \xE8 \xE9 \xEA \xEB \xEC \xED \xEE \xEF\n\0".as_ptr()
    );
}