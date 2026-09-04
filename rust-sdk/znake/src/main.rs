#![no_std]
#![no_main]

use zinux_sdk::api;

const BOX_X: i32 = 1;
const BOX_Y: i32 = 1;
const BOX_W: i32 = 54;
const BOX_H: i32 = 28;
const FW: i32 = BOX_W - 2;
const FH: i32 = BOX_H - 2;
const FX: i32 = BOX_X + 1;
const FY: i32 = BOX_Y + 1;
const PANEL_X: i32 = 57;
const PANEL_Y: i32 = 1;
const PANEL_W: i32 = 22;
const PANEL_H: i32 = 28;
const CH_HEAD: u8 = 0xFE;
const CH_BODY: u8 = 0xFE;
const CH_FOOD: u8 = b'@';
const MAX_SNAKE: usize = 512;
const SPEED_INIT: i32 = 6;
const SPEED_MIN: i32 = 2;
const SPEED_STEP: i32 = 1;

#[derive(Copy, Clone)]
struct Pt { x: i32, y: i32 }

struct Game {
    snake: [Pt; MAX_SNAKE],
    snake_len: usize,
    dx: i32, dy: i32,
    ndx: i32, ndy: i32,
    food: Pt,
    score: i32,
    spd: i32,
    dead: bool,
    rng: u32,
}

impl Game {
    fn new() -> Self {
        let mut g = Game {
            snake: [Pt { x: 0, y: 0 }; MAX_SNAKE],
            snake_len: 0,
            dx: 0, dy: 0, ndx: 0, ndy: 0,
            food: Pt { x: 0, y: 0 },
            score: 0, spd: SPEED_INIT, dead: false,
            rng: 0xBEEFCAFE,
        };
        g.game_init();
        g
    }

    fn rnd(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng
    }

    fn pat(x: i32, y: i32, ch: u8, fg: u32, bg: u32) {
        api::set_cursor(x, y); api::color(fg, bg); api::putchar(ch);
    }

    fn sat(x: i32, y: i32, s: &str, fg: u32, bg: u32) {
        api::set_cursor(x, y); api::color(fg, bg); api::print(s);
    }

    fn iat(x: i32, y: i32, v: i32, fg: u32, bg: u32) {
        api::set_cursor(x, y); api::color(fg, bg); api::print_int(v); api::print("   ");
    }

    fn wait_speed(spd: i32) {
        api::delay(40 + spd as u32 * 35);
    }

    fn draw_start() {
        api::clear();
        api::draw_window(15, 7, 50, 16, " ZNAKE ");
        Self::sat(34, 10, "* ZNAKE *", api::colors::LIGHT_GREEN, api::colors::BLUE);
        Self::sat(20, 12, "Controls:", api::colors::WHITE, api::colors::BLUE);
        Self::sat(20, 13, "  W / Up    - up", api::colors::LIGHT_GREY, api::colors::BLUE);
        Self::sat(20, 14, "  S / Down  - down", api::colors::LIGHT_GREY, api::colors::BLUE);
        Self::sat(20, 15, "  A / Left  - left", api::colors::LIGHT_GREY, api::colors::BLUE);
        Self::sat(20, 16, "  D / Right - right", api::colors::LIGHT_GREY, api::colors::BLUE);
        Self::sat(20, 18, "  ESC       - quit", api::colors::LIGHT_GREY, api::colors::BLUE);
        api::draw_button(30, 21, "[ Press any key ]", true);
    }

    fn draw_ui(&self) {
        api::clear();
        api::draw_box_titled(BOX_X, BOX_Y, BOX_W, BOX_H, "ZNAKE", api::colors::LIGHT_GREEN, api::colors::BLACK);
        api::fill_rect(FX, FY, FW, FH, b' ', api::colors::BLACK, api::colors::BLACK);
        api::draw_box_titled(PANEL_X, PANEL_Y, PANEL_W, PANEL_H, "INFO", api::colors::LIGHT_CYAN, api::colors::BLACK);

        Self::sat(PANEL_X + 2, PANEL_Y + 2, "Score:", api::colors::LIGHT_GREY, api::colors::BLACK);
        Self::sat(PANEL_X + 2, PANEL_Y + 4, "Speed:", api::colors::LIGHT_GREY, api::colors::BLACK);
        Self::sat(PANEL_X + 2, PANEL_Y + 6, "Length:", api::colors::LIGHT_GREY, api::colors::BLACK);
        api::hline(PANEL_X + 1, PANEL_Y + 8, PANEL_W - 2, api::colors::DARK_GREY, api::colors::BLACK);
        Self::sat(PANEL_X + 2, PANEL_Y + 10, "WASD/arrows", api::colors::DARK_GREY, api::colors::BLACK);
        Self::sat(PANEL_X + 2, PANEL_Y + 11, "ESC to quit", api::colors::DARK_GREY, api::colors::BLACK);
        api::hline(PANEL_X + 1, PANEL_Y + 13, PANEL_W - 2, api::colors::DARK_GREY, api::colors::BLACK);
        Self::sat(PANEL_X + 2, PANEL_Y + 15, "Eat", api::colors::LIGHT_RED, api::colors::BLACK);
        Self::pat(PANEL_X + 6, PANEL_Y + 15, CH_FOOD, api::colors::LIGHT_RED, api::colors::BLACK);
        Self::sat(PANEL_X + 8, PANEL_Y + 15, "to grow!", api::colors::LIGHT_RED, api::colors::BLACK);
    }

    fn upd_panel(&self) {
        Self::iat(PANEL_X + 10, PANEL_Y + 2, self.score, api::colors::YELLOW, api::colors::BLACK);
        Self::iat(PANEL_X + 10, PANEL_Y + 4, SPEED_INIT - self.spd + 1, api::colors::LIGHT_CYAN, api::colors::BLACK);
        Self::iat(PANEL_X + 10, PANEL_Y + 6, self.snake_len as i32, api::colors::LIGHT_GREEN, api::colors::BLACK);
    }

    fn place_food(&mut self) {
        loop {
            let fx = (self.rnd() % FW as u32) as i32;
            let fy = (self.rnd() % FH as u32) as i32;
            let mut ok = true;
            for i in 0..self.snake_len {
                if self.snake[i].x == fx && self.snake[i].y == fy { ok = false; break; }
            }
            if ok {
                self.food = Pt { x: fx, y: fy };
                Self::pat(FX + fx, FY + fy, CH_FOOD, api::colors::LIGHT_RED, api::colors::BLACK);
                return;
            }
        }
    }

    fn game_init(&mut self) {
        self.score = 0; self.spd = SPEED_INIT; self.dead = false; self.snake_len = 3;
        self.dx = 1; self.dy = 0; self.ndx = 1; self.ndy = 0;
        let cx = FW / 2; let cy = FH / 2;
        for i in 0..self.snake_len { self.snake[i] = Pt { x: cx - i as i32, y: cy }; }
        self.draw_ui();
        self.upd_panel();
        for i in (0..self.snake_len).rev() {
            Self::pat(FX + self.snake[i].x, FY + self.snake[i].y, CH_BODY, api::colors::GREEN, api::colors::BLACK);
        }
        Self::pat(FX + self.snake[0].x, FY + self.snake[0].y, CH_HEAD, api::colors::LIGHT_GREEN, api::colors::BLACK);
        self.place_food();
    }

    fn game_step(&mut self) {
        self.dx = self.ndx; self.dy = self.ndy;
        let nx = self.snake[0].x + self.dx;
        let ny = self.snake[0].y + self.dy;

        if nx < 0 || nx >= FW || ny < 0 || ny >= FH { self.dead = true; return; }
        for i in 0..self.snake_len - 1 {
            if self.snake[i].x == nx && self.snake[i].y == ny { self.dead = true; return; }
        }

        let ate = nx == self.food.x && ny == self.food.y;
        let old_tail = self.snake[self.snake_len - 1];

        if !ate { Self::pat(FX + old_tail.x, FY + old_tail.y, b' ', api::colors::BLACK, api::colors::BLACK); }

        let mut i = self.snake_len - 1;
        while i >= 1 { self.snake[i] = self.snake[i - 1]; if i == 0 { break; } i -= 1; }
        self.snake[0] = Pt { x: nx, y: ny };

        Self::pat(FX + self.snake[1].x, FY + self.snake[1].y, CH_BODY, api::colors::GREEN, api::colors::BLACK);
        Self::pat(FX + self.snake[0].x, FY + self.snake[0].y, CH_HEAD, api::colors::LIGHT_GREEN, api::colors::BLACK);

        if ate {
            if self.snake_len < MAX_SNAKE { self.snake[self.snake_len] = old_tail; self.snake_len += 1; }
            self.score += 1;
            if self.score % 5 == 0 && self.spd > SPEED_MIN { self.spd -= SPEED_STEP; }
            self.upd_panel();
            self.place_food();
        }
    }

    fn draw_gameover(&self) {
        api::draw_window(17, 10, 46, 10, " GAME OVER ");
        Self::sat(api::center_x("GAME OVER"), 12, "GAME OVER", api::colors::LIGHT_RED, api::colors::BLUE);
        Self::sat(22, 14, "Score:  ", api::colors::LIGHT_GREY, api::colors::BLUE);
        api::set_cursor(30, 14); api::color(api::colors::YELLOW, api::colors::BLUE); api::print_int(self.score);
        Self::sat(22, 15, "Length: ", api::colors::LIGHT_GREY, api::colors::BLUE);
        api::set_cursor(30, 15); api::color(api::colors::LIGHT_GREEN, api::colors::BLUE); api::print_int(self.snake_len as i32);
        api::draw_button(22, 17, "[ R ] again", true);
        api::draw_button(38, 17, "[ ESC ] quit", false);
    }

    fn handle(&mut self, k: u8) -> bool {
        if k == api::keys::UP || k == b'w' || k == b'W' { if self.dy != 1 { self.ndx = 0; self.ndy = -1; } }
        else if k == api::keys::DOWN || k == b's' || k == b'S' { if self.dy != -1 { self.ndx = 0; self.ndy = 1; } }
        else if k == api::keys::LEFT || k == b'a' || k == b'A' { if self.dx != 1 { self.ndx = -1; self.ndy = 0; } }
        else if k == api::keys::RIGHT || k == b'd' || k == b'D' { if self.dx != -1 { self.ndx = 1; self.ndy = 0; } }
        else if k == api::keys::ESC { return true; }
        false
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    Game::draw_start();
    api::waitkey();

    let mut game = Game::new();
    api::delay(50);
    api::clear_keys();

    while !game.dead {
        let mut k = 0u8;
        while api::has_key() != 0 { k = api::get_key(); }
        if k != 0 && game.handle(k) { break; }
        game.game_step();
        if !game.dead { Game::wait_speed(game.spd); }
    }

    if game.dead {
        game.draw_gameover();
        api::clear_keys();
        loop {
            let k = api::waitkey();
            if k == b'r' || k == b'R' {
                game = Game::new();
                api::delay(50);
                api::clear_keys();
                while !game.dead {
                    let mut k2 = 0u8;
                    while api::has_key() != 0 { k2 = api::get_key(); }
                    if k2 != 0 && game.handle(k2) { break; }
                    game.game_step();
                    if !game.dead { Game::wait_speed(game.spd); }
                }
                if game.dead {
                    game.draw_gameover();
                    api::clear_keys();
                    continue;
                }
                break;
            }
            if k == api::keys::ESC { break; }
        }
    }

    api::clear();
    api::color(api::colors::LIGHT_GREEN, api::colors::BLACK);
    api::print("Score: "); api::print_int(game.score); api::println("");
    zinux_sdk::syscall::syscall_exit(0)
}
