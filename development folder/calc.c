#include "zxe_api.h"

#define COLS 4
#define ROWS 5

static const char* LABELS[ROWS][COLS] = {
    { "C",   "+/-", "%",  "/" },
    { "7",   "8",   "9",  "*" },
    { "4",   "5",   "6",  "-" },
    { "1",   "2",   "3",  "+" },
    { "0",   ".",   "BS", "=" },
};

#define WIN_X   23
#define WIN_Y   2
#define WIN_W   34
#define WIN_H   20

#define DISP_X  (WIN_X + 2)
#define DISP_Y  (WIN_Y + 2)
#define DISP_W  (WIN_W - 4)

#define BTN_X   (WIN_X + 2)
#define BTN_Y   (WIN_Y + 5)
#define BTN_W   7
#define BTN_H   2

#define MAX_DISP 20

static char  disp[MAX_DISP + 2];
static int   dlen;
static int   has_dot;

static int   state;
static long  acc;
static long  saved;
static char  op;

static int   sel_row;
static int   sel_col;

static long parse(void) {
    long ip = 0, fp = 0, fdiv = 1;
    int neg = 0, i = 0, frac = 0;
    if (disp[0] == '-') { neg = 1; i = 1; }
    for (; i < dlen; i++) {
        char c = disp[i];
        if (c == '.') { frac = 1; continue; }
        if (frac) { if (fdiv < 100) { fp = fp * 10 + (c - '0'); fdiv *= 10; } }
        else      { ip = ip * 10 + (c - '0'); }
    }
    long v = ip * 100 + (fdiv == 10 ? fp * 10 : fp);
    return neg ? -v : v;
}

static void format(long v) {
    dlen = 0; has_dot = 0;
    int neg = (v < 0);
    long av = neg ? -v : v;
    long ip = av / 100;
    long fp = av % 100;

    if (neg) disp[dlen++] = '-';
    if (ip == 0) {
        disp[dlen++] = '0';
    } else {
        char tmp[12]; int tl = 0;
        for (long t = ip; t > 0; t /= 10) tmp[tl++] = '0' + (int)(t % 10);
        for (int i = tl - 1; i >= 0; i--) disp[dlen++] = tmp[i];
    }
    if (fp) {
        disp[dlen++] = '.'; has_dot = 1;
        disp[dlen++] = '0' + (int)(fp / 10);
        if (fp % 10) disp[dlen++] = '0' + (int)(fp % 10);
    }
    disp[dlen] = '\0';
}

static void reset(void) {
    disp[0] = '0'; disp[1] = '\0';
    dlen = 1; has_dot = 0;
    state = 0; acc = 0; saved = 0; op = 0;
}

static void show_err(void) {
    disp[0]='E'; disp[1]='r'; disp[2]='r'; disp[3]='\0';
    dlen = 3; state = 2; op = 0;
}

static void redraw_disp(void) {
    fill(DISP_X, DISP_Y, DISP_W, 1, ' ', BLACK, LIGHT_GREY);
    int len = slen(disp);
    int tx = DISP_X + DISP_W - len - 1;
    if (tx < DISP_X) tx = DISP_X;
    color(BLACK, LIGHT_GREY);
    gotoxy(tx, DISP_Y);
    print(disp);
    if (op && state == 1) {
        char os[2] = { op, '\0' };
        color(DARK_GREY, LIGHT_GREY);
        gotoxy(DISP_X, DISP_Y);
        print(os);
    }
}

static void redraw_btn(int r, int c, int sel) {
    int x = BTN_X + c * BTN_W;
    int y = BTN_Y + r * BTN_H;
    const char* lbl = LABELS[r][c];
    uint8_t fg, bg;

    if (lbl[0] == '=') {
        fg = WHITE; bg = sel ? LIGHT_GREEN : GREEN;
    } else if (lbl[0]=='/'||lbl[0]=='*'||lbl[0]=='-'||
               (lbl[0]=='+'&&lbl[1]=='\0')) {
        fg = WHITE; bg = sel ? YELLOW : BROWN;
    } else if (lbl[0]=='C'||lbl[0]=='B'||(lbl[0]=='+'&&lbl[1]=='/')) {
        fg = BLACK; bg = sel ? LIGHT_RED : RED;
        if (lbl[0]=='B') { bg = sel ? WHITE : LIGHT_GREY; }
    } else {
        fg = BLACK; bg = sel ? WHITE : LIGHT_GREY;
    }

    color(DARK_GREY, BLACK);
    gotoxy(x + 1, y + 1);
    print("      ");

    fill(x, y, BTN_W - 1, 1, ' ', fg, bg);
    int lx = x + (BTN_W - 1 - slen(lbl)) / 2;
    gotoxy(lx, y);
    color(fg, bg);
    print(lbl);
}

static void redraw_all(void) {
    clear();
    window(WIN_X, WIN_Y, WIN_W, WIN_H, " CALCULATOR ");
    color(DARK_GREY, BLUE);
    gotoxy(WIN_X + 2, WIN_Y + WIN_H - 2);
    print("arrows+Enter  or  keyboard");
    redraw_disp();
    hline(WIN_X + 1, WIN_Y + 4, WIN_W - 2, DARK_GREY, BLUE);
    for (int rr = 0; rr < ROWS; rr++)
        for (int cc = 0; cc < COLS; cc++)
            redraw_btn(rr, cc, rr == sel_row && cc == sel_col);
}

static void redraw_btns(void) {
    for (int rr = 0; rr < ROWS; rr++)
        for (int cc = 0; cc < COLS; cc++)
            redraw_btn(rr, cc, rr == sel_row && cc == sel_col);
}

static void do_calc(void) {
    long a = saved, b = acc, res = 0;
    if      (op == '+') res = a + b;
    else if (op == '-') res = a - b;
    else if (op == '*') res = (a / 100) * b + (a % 100) * b / 100;
    else if (op == '/') {
        if (b == 0) { show_err(); return; }
        res = (a * 100) / b;
    } else { res = b; }
    acc = res;
    format(res);
}

static void press(int r, int c) {
    const char* lbl = LABELS[r][c];
    char ch = lbl[0];

    if (ch == 'C') { reset(); redraw_disp(); return; }

    if (ch >= '0' && ch <= '9') {
        if (state == 1) {
            disp[0] = '0'; disp[1] = '\0'; dlen = 1; has_dot = 0;
            state = 1;
            state = 2;
            dlen = 0;
        } else if (state == 2 && op == 0) {
            reset();
            dlen = 0;
        }
        if (dlen == 1 && disp[0] == '0') dlen = 0;
        if (dlen < MAX_DISP) { disp[dlen++] = ch; disp[dlen] = '\0'; }
        if (state == 1) state = 2;
        acc = parse();
        redraw_disp();
        return;
    }

    if (ch == '.') {
        if (state == 1) { dlen = 0; disp[0]='\0'; state = 2; }
        else if (state == 2 && op == 0) { reset(); dlen = 0; }
        if (!has_dot) {
            if (dlen == 0) { disp[dlen++] = '0'; }
            disp[dlen++] = '.'; disp[dlen] = '\0'; has_dot = 1;
        }
        acc = parse();
        redraw_disp();
        return;
    }

    if (ch == 'B') {
        if (state == 2 && op == 0) { reset(); redraw_disp(); return; }
        if (dlen > 1) {
            if (disp[dlen-1] == '.') has_dot = 0;
            disp[--dlen] = '\0';
        } else {
            disp[0] = '0'; disp[1] = '\0'; dlen = 1; has_dot = 0;
        }
        acc = parse();
        redraw_disp();
        return;
    }

    if (ch == '+' && lbl[1] == '/') {
        if (disp[0] == '-') {
            for (int i = 0; i < dlen; i++) disp[i] = disp[i+1];
            dlen--;
        } else if (dlen < MAX_DISP) {
            for (int i = dlen; i >= 0; i--) disp[i+1] = disp[i];
            disp[0] = '-'; dlen++;
        }
        acc = parse();
        redraw_disp();
        return;
    }

    if (ch == '%') {
        acc = parse() / 100;
        format(acc);
        state = 2; op = 0;
        redraw_disp();
        return;
    }

    if (ch=='+' || ch=='-' || ch=='*' || ch=='/') {
        if (state == 1) {
            op = ch;
            redraw_disp();
            return;
        }
        if (state == 0 || state == 2) {
            if (op != 0 && state == 2) {
                do_calc();
            } else {
                acc = parse();
            }
            saved = acc;
        }
        op = ch;
        state = 1;
        redraw_disp();
        return;
    }

    if (ch == '=') {
        if (op == 0) return;
        if (state == 1) {
            acc = saved;
        }
        do_calc();
        op = 0; state = 2;
        redraw_disp();
        return;
    }
}

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;

    reset();
    sel_row = 4; sel_col = 3;
    redraw_all();

    while (1) {
        char k = waitkey();
        unsigned char uk = (unsigned char)k;

        int moved = 0;
        if      (uk == 128) { sel_row = sel_row > 0 ? sel_row-1 : ROWS-1; moved=1; }
        else if (uk == 129) { sel_row = sel_row < ROWS-1 ? sel_row+1 : 0; moved=1; }
        else if (uk == 130) { sel_col = sel_col > 0 ? sel_col-1 : COLS-1; moved=1; }
        else if (uk == 131) { sel_col = sel_col < COLS-1 ? sel_col+1 : 0; moved=1; }
        else if (k == '\n' || k == ' ') {
            press(sel_row, sel_col);
            redraw_btns();
            continue;
        }
        else if (k == 27) break;
        else {
            int r = -1, c = -1;
            if (k >= '0' && k <= '9') {
                for (int rr=0;rr<ROWS;rr++) for (int cc=0;cc<COLS;cc++)
                    if (LABELS[rr][cc][0]==k && LABELS[rr][cc][1]=='\0') { r=rr; c=cc; }
            } else if (k=='+') { r=3;c=3; }
            else if (k=='-')   { r=2;c=3; }
            else if (k=='*')   { r=1;c=3; }
            else if (k=='/')   { r=0;c=3; }
            else if (k=='=')   { r=4;c=3; }
            else if (k=='.')   { r=4;c=1; }
            else if (k=='\b')  { r=4;c=2; }
            else if (k=='c'||k=='C') { r=0;c=0; }
            if (r >= 0) { sel_row=r; sel_col=c; press(r,c); redraw_btns(); continue; }
        }

        if (moved) redraw_btns();
    }

    clear();
    return 0;
}