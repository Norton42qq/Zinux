#![no_std]
#![no_main]

use zinux_sdk::api;

const C_BG: u32 = api::colors::rgb(30, 30, 40);
const C_WIN_BG: u32 = api::colors::rgb(45, 45, 60);
const C_WIN_BORDER: u32 = api::colors::rgb(80, 80, 120);
const C_DISP_BG: u32 = api::colors::rgb(15, 20, 15);
const C_DISP_FG: u32 = api::colors::rgb(80, 255, 80);
const C_DISP_SMALL: u32 = api::colors::rgb(40, 140, 40);
const C_BTN_NUM_BG: u32 = api::colors::rgb(60, 60, 80);
const C_BTN_NUM_HL: u32 = api::colors::rgb(90, 90, 120);
const C_BTN_NUM_PR: u32 = api::colors::rgb(40, 40, 55);
const C_BTN_OP_BG: u32 = api::colors::rgb(80, 60, 20);
const C_BTN_OP_HL: u32 = api::colors::rgb(120, 90, 30);
const C_BTN_EQ_BG: u32 = api::colors::rgb(20, 100, 30);
const C_BTN_EQ_HL: u32 = api::colors::rgb(30, 160, 50);
const C_BTN_CLR_BG: u32 = api::colors::rgb(120, 30, 30);
const C_BTN_CLR_HL: u32 = api::colors::rgb(180, 50, 50);
const C_BTN_FG: u32 = api::colors::rgb(240, 240, 240);
const C_BTN_SHADOW: u32 = api::colors::rgb(15, 15, 20);
const C_TITLE_FG: u32 = api::colors::rgb(160, 200, 255);
const C_SEP: u32 = api::colors::rgb(60, 60, 80);
const C_HINT: u32 = api::colors::rgb(90, 90, 110);

const WIN_W: i32 = 340;
const WIN_H: i32 = 480;
const TITLE_H: i32 = 28;
const DISP_H: i32 = 80;
const DISP_PAD: i32 = 10;
const BTN_COLS: i32 = 4;
const BTN_ROWS: i32 = 5;
const BTN_PAD: i32 = 6;
const MAX_DISP: usize = 16;

#[derive(Clone, Copy, PartialEq)]
enum BtnKind { Num, Op, Eq, Clr, Fn }

struct Btn {
    label: &'static str,
    kind: BtnKind,
}

const BUTTONS: [[Btn; 4]; 5] = [
    [Btn { label: "C", kind: BtnKind::Clr }, Btn { label: "+/-", kind: BtnKind::Fn }, Btn { label: "%", kind: BtnKind::Fn }, Btn { label: "/", kind: BtnKind::Op }],
    [Btn { label: "7", kind: BtnKind::Num }, Btn { label: "8", kind: BtnKind::Num }, Btn { label: "9", kind: BtnKind::Num }, Btn { label: "*", kind: BtnKind::Op }],
    [Btn { label: "4", kind: BtnKind::Num }, Btn { label: "5", kind: BtnKind::Num }, Btn { label: "6", kind: BtnKind::Num }, Btn { label: "-", kind: BtnKind::Op }],
    [Btn { label: "1", kind: BtnKind::Num }, Btn { label: "2", kind: BtnKind::Num }, Btn { label: "3", kind: BtnKind::Num }, Btn { label: "+", kind: BtnKind::Op }],
    [Btn { label: "0", kind: BtnKind::Num }, Btn { label: ".", kind: BtnKind::Fn }, Btn { label: "<", kind: BtnKind::Fn }, Btn { label: "=", kind: BtnKind::Eq }],
];

struct Calc {
    disp: [u8; MAX_DISP + 2],
    dlen: usize,
    has_dot: bool,
    state: i32,
    acc: i64,
    saved: i64,
    op_cur: u8,
    sel_r: i32,
    sel_c: i32,
    win_x: i32,
    win_y: i32,
    btn_w: i32,
    btn_h: i32,
}

impl Calc {
    fn new() -> Self {
        let mut c = Calc {
            disp: [0u8; MAX_DISP + 2],
            dlen: 0,
            has_dot: false,
            state: 0,
            acc: 0,
            saved: 0,
            op_cur: 0,
            sel_r: 4,
            sel_c: 3,
            win_x: 0,
            win_y: 0,
            btn_w: 0,
            btn_h: 0,
        };
        c.do_reset();
        let sw = api::screen_w();
        let sh = api::screen_h();
        c.win_x = (sw - WIN_W) / 2;
        c.win_y = (sh - WIN_H) / 2;
        c.btn_w = (WIN_W - BTN_PAD * (BTN_COLS + 1)) / BTN_COLS;
        c.btn_h = (WIN_H - TITLE_H - DISP_H - BTN_PAD * (BTN_ROWS + 1) - 20) / BTN_ROWS;
        c
    }

    fn do_parse(&self) -> i64 {
        let mut ip: i64 = 0;
        let mut fp: i64 = 0;
        let mut fdiv: i64 = 1;
        let mut neg = false;
        let mut i = 0;
        let s = &self.disp[..self.dlen];
        if s.len() > 0 && s[0] == b'-' { neg = true; i = 1; }
        let mut frac = false;
        while i < s.len() {
            let c = s[i];
            if c == b'.' { frac = true; i += 1; continue; }
            let d = (c - b'0') as i64;
            if frac {
                if fdiv < 10 { fp = fp * 10 + d; fdiv *= 10; }
            } else {
                ip = ip * 10 + d;
            }
            i += 1;
        }
        let v = ip * 100 + if fdiv == 10 { fp * 10 } else { fp };
        if neg { -v } else { v }
    }

    fn do_format(&mut self, mut v: i64) {
        self.dlen = 0;
        self.has_dot = false;
        let neg = v < 0;
        if neg { v = -v; }
        let ip = v / 100;
        let fp = v % 100;
        if neg { self.disp[self.dlen] = b'-'; self.dlen += 1; }
        if ip == 0 {
            self.disp[self.dlen] = b'0'; self.dlen += 1;
        } else {
            let mut tmp = [0u8; 12];
            let mut tl = 0;
            let mut t = ip;
            while t > 0 { tmp[tl] = b'0' + (t % 10) as u8; t /= 10; tl += 1; }
            let mut i = tl;
            while i > 0 { i -= 1; self.disp[self.dlen] = tmp[i]; self.dlen += 1; }
        }
        if fp != 0 {
            self.disp[self.dlen] = b'.'; self.dlen += 1; self.has_dot = true;
            self.disp[self.dlen] = b'0' + (fp / 10) as u8; self.dlen += 1;
            if fp % 10 != 0 {
                self.disp[self.dlen] = b'0' + (fp % 10) as u8; self.dlen += 1;
            }
        }
        self.disp[self.dlen] = 0;
    }

    fn do_reset(&mut self) {
        self.disp[0] = b'0'; self.disp[1] = 0;
        self.dlen = 1; self.has_dot = false;
        self.state = 0; self.acc = 0; self.saved = 0; self.op_cur = 0;
    }

    fn do_err(&mut self) {
        self.disp[0] = b'E'; self.disp[1] = b'r'; self.disp[2] = b'r'; self.disp[3] = 0;
        self.dlen = 3; self.state = 2; self.op_cur = 0;
    }

    fn rounded_rect(&self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        api::gfx_fillrect(x + 1, y, w - 2, h, c);
        api::gfx_fillrect(x, y + 1, w, h - 2, c);
    }

    fn centered_text(&self, x: i32, y: i32, w: i32, h: i32, s: &str, fg: u32, bg: u32) {
        let len = api::slen(s);
        let tx = x + (w - len * 8) / 2;
        let ty = y + (h - 8) / 2;
        if bg != C_BG { api::gfx_fillrect(x, y, w, h, bg); }
        for (i, &ch) in s.as_bytes().iter().enumerate() {
            api::gfx_char(tx + i as i32 * 8, ty, ch, fg, bg);
        }
    }

    fn draw_shadow(&self, x: i32, y: i32, w: i32, h: i32) {
        api::gfx_fillrect(x + 3, y + 3, w, h, C_BTN_SHADOW);
    }

    fn draw_button_at(&self, r: i32, c: i32, pressed: bool) {
        let x = self.win_x + BTN_PAD + c * (self.btn_w + BTN_PAD);
        let y = self.win_y + TITLE_H + DISP_H + BTN_PAD + r * (self.btn_h + BTN_PAD);
        let is_sel = r == self.sel_r && c == self.sel_c;
        let k = &BUTTONS[r as usize][c as usize].kind;
        let (bg, bg_hl) = match k {
            BtnKind::Clr => (C_BTN_CLR_BG, C_BTN_CLR_HL),
            BtnKind::Op => (C_BTN_OP_BG, C_BTN_OP_HL),
            BtnKind::Eq => (C_BTN_EQ_BG, C_BTN_EQ_HL),
            _ => (C_BTN_NUM_BG, C_BTN_NUM_HL),
        };
        let use_bg = if pressed { C_BTN_NUM_PR } else if is_sel { bg_hl } else { bg };
        if !pressed { self.draw_shadow(x, y, self.btn_w, self.btn_h); }
        self.rounded_rect(x, y, self.btn_w, self.btn_h, use_bg);
        if !pressed {
            api::gfx_fillrect(x + 1, y + 1, self.btn_w - 2, 2, api::colors::rgb(255, 255, 255));
        }
        self.centered_text(x, y, self.btn_w, self.btn_h, BUTTONS[r as usize][c as usize].label, C_BTN_FG, use_bg);
        if is_sel && !pressed {
            api::gfx_rect(x, y, self.btn_w, self.btn_h, api::colors::rgb(255, 255, 150));
        }
    }

    fn draw_display(&self) {
        let dx = self.win_x + DISP_PAD;
        let dy = self.win_y + TITLE_H + DISP_PAD;
        let dw = WIN_W - DISP_PAD * 2;
        let dh = DISP_H - DISP_PAD * 2;
        self.rounded_rect(dx, dy, dw, dh, C_DISP_BG);
        api::gfx_rect(dx, dy, dw, dh, C_WIN_BORDER);
        if self.op_cur != 0 && self.state >= 1 {
            api::gfx_char(dx + 6, dy + 6, self.op_cur, C_DISP_SMALL, C_DISP_BG);
        }
        let len = self.dlen as i32;
        let char_w: i32 = if self.dlen > 10 { 12 } else { 20 };
        let text_w = len * char_w;
        let mut tx = dx + dw - text_w - 8;
        let ty = dy + dh / 2 - 10;
        if tx < dx + 4 { tx = dx + 4; }
        let y_off = if char_w == 20 { 4 } else { 8 };
        for i in 0..self.dlen {
            api::gfx_char(tx + i as i32 * char_w, ty + y_off, self.disp[i], C_DISP_FG, C_DISP_BG);
        }
    }

    fn draw_titlebar(&self) {
        api::gfx_fillrect(self.win_x, self.win_y, WIN_W, TITLE_H, C_WIN_BORDER);
        let bx = self.win_x + WIN_W - 22;
        let by = self.win_y + 6;
        api::gfx_fillrect(bx, by, 16, 16, C_BTN_CLR_BG);
        api::gfx_char(bx + 4, by + 4, b'X', C_BTN_FG, C_BTN_CLR_BG);
        let title = " ZALCULATOR ";
        let tlen = api::slen(title);
        let tx = self.win_x + (WIN_W - tlen * 8) / 2;
        for (i, &ch) in title.as_bytes().iter().enumerate() {
            api::gfx_char(tx + i as i32 * 8, self.win_y + 10, ch, C_TITLE_FG, C_WIN_BORDER);
        }
    }

    fn draw_all(&self) {
        api::gfx_fillrect(0, 0, api::screen_w(), api::screen_h(), C_BG);
        api::gfx_fillrect(self.win_x + 6, self.win_y + 6, WIN_W, WIN_H, C_BTN_SHADOW);
        self.rounded_rect(self.win_x, self.win_y, WIN_W, WIN_H, C_WIN_BG);
        api::gfx_rect(self.win_x, self.win_y, WIN_W, WIN_H, C_WIN_BORDER);
        self.draw_titlebar();
        api::gfx_fillrect(self.win_x + BTN_PAD, self.win_y + TITLE_H + DISP_H - 2, WIN_W - BTN_PAD * 2, 1, C_SEP);
        self.draw_display();
        for r in 0..BTN_ROWS {
            for c in 0..BTN_COLS {
                self.draw_button_at(r, c, false);
            }
        }
        let hint = "arrows+Enter / keyboard / ESC";
        let hlen = api::slen(hint);
        let hx = self.win_x + (WIN_W - hlen * 8) / 2;
        let hy = self.win_y + WIN_H - 14;
        for (i, &ch) in hint.as_bytes().iter().enumerate() {
            api::gfx_char(hx + i as i32 * 8, hy, ch, C_HINT, C_WIN_BG);
        }
    }

    fn redraw_btns(&self) {
        for r in 0..BTN_ROWS {
            for c in 0..BTN_COLS {
                self.draw_button_at(r, c, false);
            }
        }
    }

    fn calc_press(&mut self, r: i32, c: i32) {
        let lbl = BUTTONS[r as usize][c as usize].label;
        let ch = lbl.as_bytes()[0];

        if ch == b'C' { self.do_reset(); return; }

        if ch >= b'0' && ch <= b'9' {
            if self.state == 1 { self.disp[0] = b'0'; self.disp[1] = 0; self.dlen = 1; self.has_dot = false; self.state = 2; self.dlen = 0; }
            else if self.state == 2 && self.op_cur == 0 { self.do_reset(); self.dlen = 0; }
            if self.dlen == 1 && self.disp[0] == b'0' { self.dlen = 0; }
            if self.dlen < MAX_DISP { self.disp[self.dlen] = ch; self.dlen += 1; self.disp[self.dlen] = 0; }
            if self.state == 1 { self.state = 2; }
            self.acc = self.do_parse();
            return;
        }

        if ch == b'.' {
            if self.state == 1 { self.dlen = 0; self.disp[0] = 0; self.state = 2; }
            else if self.state == 2 && self.op_cur == 0 { self.do_reset(); self.dlen = 0; }
            if !self.has_dot {
                if self.dlen == 0 { self.disp[0] = b'0'; self.dlen = 1; }
                self.disp[self.dlen] = b'.'; self.dlen += 1; self.disp[self.dlen] = 0; self.has_dot = true;
            }
            self.acc = self.do_parse();
            return;
        }

        if ch == b'<' {
            if self.state == 2 && self.op_cur == 0 { self.do_reset(); return; }
            if self.dlen > 1 {
                if self.disp[self.dlen - 1] == b'.' { self.has_dot = false; }
                self.dlen -= 1;
                self.disp[self.dlen] = 0;
            } else { self.disp[0] = b'0'; self.disp[1] = 0; self.dlen = 1; self.has_dot = false; }
            self.acc = self.do_parse();
            return;
        }

        if ch == b'+' && lbl.len() > 1 && lbl.as_bytes()[1] == b'/' {
            if self.disp[0] == b'-' {
                let mut i = 0;
                while i < self.dlen { self.disp[i] = self.disp[i + 1]; i += 1; }
                self.dlen -= 1;
            } else if self.dlen < MAX_DISP {
                let mut i = self.dlen;
                while i > 0 { self.disp[i] = self.disp[i - 1]; i -= 1; }
                self.disp[0] = b'-'; self.dlen += 1;
            }
            self.acc = self.do_parse();
            return;
        }

        if ch == b'%' {
            self.acc = self.do_parse() / 100;
            self.do_format(self.acc);
            self.state = 2; self.op_cur = 0;
            return;
        }

        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'/' {
            if self.state == 1 { self.op_cur = ch; return; }
            if self.op_cur != 0 && self.state == 2 {
                let a = self.saved;
                let b = self.acc;
                let res = match self.op_cur {
                    b'+' => a + b,
                    b'-' => a - b,
                    b'*' => (a / 100) * b + (a % 100) * b / 100,
                    b'/' => { if b == 0 { self.do_err(); return; } (a * 100) / b }
                    _ => 0,
                };
                self.acc = res;
                self.do_format(res);
            } else { self.acc = self.do_parse(); }
            self.saved = self.acc; self.op_cur = ch; self.state = 1;
            return;
        }

        if ch == b'=' {
            if self.op_cur == 0 { return; }
            if self.state == 1 { self.acc = self.saved; }
            let a = self.saved;
            let b = self.acc;
            let res = match self.op_cur {
                b'+' => a + b,
                b'-' => a - b,
                b'*' => (a / 100) * b + (a % 100) * b / 100,
                b'/' => { if b == 0 { self.do_err(); return; } (a * 100) / b }
                _ => 0,
            };
            self.acc = res;
            self.do_format(res);
            self.op_cur = 0; self.state = 2;
        }
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    let mut calc = Calc::new();
    calc.draw_all();

    loop {
        let k = api::waitkey();
        let mut moved = false;

        if k == api::keys::UP { calc.sel_r = if calc.sel_r > 0 { calc.sel_r - 1 } else { BTN_ROWS - 1 }; moved = true; }
        else if k == api::keys::DOWN { calc.sel_r = if calc.sel_r < BTN_ROWS - 1 { calc.sel_r + 1 } else { 0 }; moved = true; }
        else if k == api::keys::LEFT { calc.sel_c = if calc.sel_c > 0 { calc.sel_c - 1 } else { BTN_COLS - 1 }; moved = true; }
        else if k == api::keys::RIGHT { calc.sel_c = if calc.sel_c < BTN_COLS - 1 { calc.sel_c + 1 } else { 0 }; moved = true; }
        else if k == api::keys::ENTER || k == api::keys::SPACE {
            calc.draw_button_at(calc.sel_r, calc.sel_c, true);
            api::delay(80);
            calc.calc_press(calc.sel_r, calc.sel_c);
            calc.draw_display();
            calc.draw_button_at(calc.sel_r, calc.sel_c, false);
            continue;
        }
        else if k == api::keys::ESC { break; }
        else {
            let mut r: i32 = -1;
            let mut c: i32 = -1;
            if k >= b'0' && k <= b'9' {
                for rr in 0..BTN_ROWS {
                    for cc in 0..BTN_COLS {
                        if BUTTONS[rr as usize][cc as usize].label.as_bytes()[0] == k
                            && BUTTONS[rr as usize][cc as usize].label.len() == 1 {
                            r = rr; c = cc;
                        }
                    }
                }
            }
            else if k == b'+' { r = 3; c = 3; }
            else if k == b'-' { r = 2; c = 3; }
            else if k == b'*' { r = 1; c = 3; }
            else if k == b'/' { r = 0; c = 3; }
            else if k == b'=' { r = 4; c = 3; }
            else if k == b'.' { r = 4; c = 1; }
            else if k == api::keys::BACKSPACE { r = 4; c = 2; }
            else if k == b'c' || k == b'C' { r = 0; c = 0; }
            if r >= 0 {
                let old_sr = calc.sel_r;
                let old_sc = calc.sel_c;
                calc.draw_button_at(r, c, true);
                api::delay(80);
                calc.calc_press(r, c);
                calc.draw_display();
                calc.sel_r = r; calc.sel_c = c;
                calc.draw_button_at(r, c, false);
                calc.sel_r = old_sr; calc.sel_c = old_sc;
                calc.draw_button_at(r, c, false);
                continue;
            }
        }

        if moved { calc.redraw_btns(); }
    }

    api::clear();
    zinux_sdk::syscall::syscall_exit(0)
}
