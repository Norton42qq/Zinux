#![no_std]
#![no_main]

use zinux_sdk::api;

const BOARD_W: i32 = 10;
const BOARD_H: i32 = 20;
const BLOCK_SIZE: i32 = 30;
const WIN_W: i32 = BOARD_W * BLOCK_SIZE + 160;
const WIN_H: i32 = BOARD_H * BLOCK_SIZE;
const BG_COLOR: u32 = api::colors::DARK_GREY;
const BOARD_BG: u32 = api::colors::BLACK;
const BORDER_COLOR: u32 = api::colors::WHITE;

const TETROMINOS: [[[i32; 4]; 4]; 7] = [
    [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]],
    [[0,0,0,0],[0,1,1,0],[0,1,1,0],[0,0,0,0]],
    [[0,0,0,0],[0,1,0,0],[1,1,1,0],[0,0,0,0]],
    [[0,0,0,0],[0,1,1,0],[1,1,0,0],[0,0,0,0]],
    [[0,0,0,0],[1,1,0,0],[0,1,1,0],[0,0,0,0]],
    [[0,0,0,0],[1,0,0,0],[1,1,1,0],[0,0,0,0]],
    [[0,0,0,0],[0,0,1,0],[1,1,1,0],[0,0,0,0]],
];

const COLORS: [u32; 7] = [
    api::colors::CYAN, api::colors::YELLOW, api::colors::MAGENTA,
    api::colors::LIGHT_GREEN, api::colors::LIGHT_RED, api::colors::BROWN, api::colors::LIGHT_BLUE,
];

struct Tetris {
    board: [[i32; 10]; 20],
    cur_x: i32, cur_y: i32, cur_type: usize, cur_rot: i32,
    next_type: usize,
    score: i32,
    game_over: bool,
    rnd_seed: u32,
}

impl Tetris {
    fn new() -> Self {
        let mut t = Tetris {
            board: [[0; 10]; 20],
            cur_x: 0, cur_y: 0, cur_type: 0, cur_rot: 0,
            next_type: 0,
            score: 0, game_over: false,
            rnd_seed: 1,
        };
        let (h, m, s) = api::get_time();
        t.rnd_seed = (h as u32 * 3600 + m as u32 * 60 + s as u32) * 1000;
        t.next_type = (t.rand() % 7) as usize;
        t.spawn_piece();
        t
    }

    fn rand(&mut self) -> i32 {
        self.rnd_seed = self.rnd_seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rnd_seed / 65536) % 32768) as i32
    }

    fn draw_block(x: i32, y: i32, col: u32) {
        let px = x * BLOCK_SIZE;
        let py = y * BLOCK_SIZE;
        api::gfx_fillrect(px, py, BLOCK_SIZE - 1, BLOCK_SIZE - 1, col);
        api::gfx_rect(px, py, BLOCK_SIZE - 1, BLOCK_SIZE - 1, BORDER_COLOR);
    }

    fn draw_board(&self) {
        api::gfx_fillrect(0, 0, BOARD_W * BLOCK_SIZE, BOARD_H * BLOCK_SIZE, BOARD_BG);
        for y in 0..BOARD_H {
            for x in 0..BOARD_W {
                if self.board[y as usize][x as usize] != 0 {
                    Self::draw_block(x, y, COLORS[self.board[y as usize][x as usize] as usize - 1]);
                }
            }
        }
    }

    fn get_rotated(type_idx: usize, rot: i32) -> [[i32; 4]; 4] {
        let mut out = [[0i32; 4]; 4];
        for i in 0..4 { for j in 0..4 { out[i][j] = TETROMINOS[type_idx][i][j]; } }
        for _ in 0..rot {
            let mut tmp = [[0i32; 4]; 4];
            for i in 0..4 { for j in 0..4 { tmp[j][3 - i] = out[i][j]; } }
            out = tmp;
        }
        out
    }

    fn collides(&self, tx: i32, ty: i32, type_idx: usize, rot: i32) -> bool {
        let shape = Self::get_rotated(type_idx, rot);
        for y in 0..4 {
            for x in 0..4 {
                if shape[y][x] != 0 {
                    let bx = tx + x as i32;
                    let by = ty + y as i32;
                    if bx < 0 || bx >= BOARD_W || by >= BOARD_H || by < 0 { return true; }
                    if by >= 0 && self.board[by as usize][bx as usize] != 0 { return true; }
                }
            }
        }
        false
    }

    fn merge_piece(&mut self) {
        let shape = Self::get_rotated(self.cur_type, self.cur_rot);
        for y in 0..4 {
            for x in 0..4 {
                if shape[y][x] != 0 {
                    let bx = self.cur_x + x as i32;
                    let by = self.cur_y + y as i32;
                    if by >= 0 && by < BOARD_H && bx >= 0 && bx < BOARD_W {
                        self.board[by as usize][bx as usize] = self.cur_type as i32 + 1;
                    }
                }
            }
        }
    }

    fn clear_lines(&mut self) {
        let mut lines = 0i32;
        let mut y = BOARD_H - 1;
        loop {
            if y < 0 { break; }
            let mut full = true;
            for x in 0..BOARD_W {
                if self.board[y as usize][x as usize] == 0 { full = false; break; }
            }
            if full {
                for yy in (1..=y).rev() {
                    for x in 0..BOARD_W {
                        self.board[yy as usize][x as usize] = self.board[(yy - 1) as usize][x as usize];
                    }
                }
                for x in 0..BOARD_W { self.board[0][x as usize] = 0; }
                lines += 1;
            } else {
                y -= 1;
            }
        }
        if lines > 0 { self.score += lines * 100; }
    }

    fn spawn_piece(&mut self) {
        self.cur_type = self.next_type;
        self.next_type = (self.rand() % 7) as usize;
        self.cur_rot = 0;
        self.cur_x = BOARD_W / 2 - 2;
        self.cur_y = 0;
        if self.collides(self.cur_x, self.cur_y, self.cur_type, self.cur_rot) {
            self.game_over = true;
        }
    }

    fn rotate_piece(&mut self) {
        let new_rot = (self.cur_rot + 1) % 4;
        if !self.collides(self.cur_x, self.cur_y, self.cur_type, new_rot) {
            self.cur_rot = new_rot;
        }
    }

    fn draw_current(&self) {
        let shape = Self::get_rotated(self.cur_type, self.cur_rot);
        for y in 0..4 {
            for x in 0..4 {
                if shape[y][x] != 0 {
                    Self::draw_block(self.cur_x + x as i32, self.cur_y + y as i32, COLORS[self.cur_type]);
                }
            }
        }
    }

    fn draw_next(&self) {
        let ox = BOARD_W * BLOCK_SIZE + 20;
        let oy = 50;
        api::gfx_fillrect(ox, oy, 4 * BLOCK_SIZE, 4 * BLOCK_SIZE, api::colors::BLACK);
        api::gfx_rect(ox, oy, 4 * BLOCK_SIZE, 4 * BLOCK_SIZE, api::colors::WHITE);
        api::gfx_text(ox, oy - 20, "Next:", api::colors::WHITE, BG_COLOR);
        let shape = Self::get_rotated(self.next_type, 0);
        for y in 0..4 {
            for x in 0..4 {
                if shape[y][x] != 0 {
                    let px = ox + x as i32 * BLOCK_SIZE;
                    let py = oy + y as i32 * BLOCK_SIZE;
                    api::gfx_fillrect(px, py, BLOCK_SIZE - 1, BLOCK_SIZE - 1, COLORS[self.next_type]);
                    api::gfx_rect(px, py, BLOCK_SIZE - 1, BLOCK_SIZE - 1, api::colors::WHITE);
                }
            }
        }
    }

    fn draw_score(&self) {
        let mut buf = [0u8; 16];
        api::int_to_str(self.score, &mut buf);
        let s = core::str::from_utf8(&buf).unwrap_or("0");
        api::gfx_text(BOARD_W * BLOCK_SIZE + 20, 200, "Score:", api::colors::WHITE, BG_COLOR);
        api::gfx_text(BOARD_W * BLOCK_SIZE + 20, 220, s, api::colors::YELLOW, BG_COLOR);
    }

    fn redraw_all(&self) {
        self.draw_board();
        self.draw_current();
        self.draw_next();
        self.draw_score();
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    api::gfx_fillrect(0, 0, WIN_W, WIN_H, BG_COLOR);
    api::gfx_text(10, 10, "TETRIS - Arrow keys, Space = drop, ESC = exit", api::colors::WHITE, BG_COLOR);

    let mut game = Tetris::new();
    let mut fall_counter = 0;

    while !game.game_over {
        game.redraw_all();

        if api::has_key() != 0 {
            let key = api::get_key();
            if key == api::keys::LEFT {
                if !game.collides(game.cur_x - 1, game.cur_y, game.cur_type, game.cur_rot) { game.cur_x -= 1; }
            } else if key == api::keys::RIGHT {
                if !game.collides(game.cur_x + 1, game.cur_y, game.cur_type, game.cur_rot) { game.cur_x += 1; }
            } else if key == api::keys::DOWN {
                if !game.collides(game.cur_x, game.cur_y + 1, game.cur_type, game.cur_rot) {
                    game.cur_y += 1;
                } else {
                    game.merge_piece(); game.clear_lines(); game.spawn_piece();
                }
            } else if key == api::keys::UP {
                game.rotate_piece();
            } else if key == api::keys::SPACE {
                while !game.collides(game.cur_x, game.cur_y + 1, game.cur_type, game.cur_rot) { game.cur_y += 1; }
                game.merge_piece(); game.clear_lines(); game.spawn_piece();
            } else if key == api::keys::ESC { break; }
        }

        fall_counter += 1;
        if fall_counter >= 8 {
            fall_counter = 0;
            if !game.collides(game.cur_x, game.cur_y + 1, game.cur_type, game.cur_rot) {
                game.cur_y += 1;
            } else {
                game.merge_piece(); game.clear_lines(); game.spawn_piece();
                if game.game_over { break; }
            }
        }
        api::delay(1000);
    }

    api::gfx_fillrect(0, 0, WIN_W, WIN_H, BG_COLOR);
    api::gfx_text(WIN_W / 2 - 50, WIN_H / 2, "Игра окончена", api::colors::RED, BG_COLOR);
    let mut buf = [0u8; 16];
    api::int_to_str(game.score, &mut buf);
    let s = core::str::from_utf8(&buf).unwrap_or("0");
    api::gfx_text(WIN_W / 2 - 40, WIN_H / 2 + 30, "Счет: ", api::colors::WHITE, BG_COLOR);
    api::gfx_text(WIN_W / 2 + 20, WIN_H / 2 + 30, s, api::colors::YELLOW, BG_COLOR);
    api::gfx_text(WIN_W / 2 - 120, WIN_H / 2 + 60, "Нажмите любую клавишу для выхода", api::colors::LIGHT_CYAN, BG_COLOR);
    api::waitkey();
    api::clear();
    zinux_sdk::syscall::syscall_exit(0)
}
