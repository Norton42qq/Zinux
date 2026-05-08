#include "zxe_api.h"

// Цвета
#define C_BG          RGB(30,  30,  40)
#define C_WIN_BG      RGB(45,  45,  60)
#define C_WIN_BORDER  RGB(80,  80, 120)
#define C_DISP_BG     RGB(15,  20,  15)
#define C_DISP_FG     RGB(80, 255,  80)
#define C_DISP_SMALL  RGB(40, 140,  40)
#define C_BTN_NUM_BG  RGB(60,  60,  80)
#define C_BTN_NUM_HL  RGB(90,  90, 120)
#define C_BTN_NUM_PR  RGB(40,  40,  55)
#define C_BTN_OP_BG   RGB(80,  60,  20)
#define C_BTN_OP_HL   RGB(120, 90,  30)
#define C_BTN_EQ_BG   RGB(20,  100, 30)
#define C_BTN_EQ_HL   RGB(30,  160, 50)
#define C_BTN_CLR_BG  RGB(120, 30,  30)
#define C_BTN_CLR_HL  RGB(180, 50,  50)
#define C_BTN_FG      RGB(240, 240, 240)
#define C_BTN_SHADOW  RGB(15,  15,  20)
#define C_TITLE_FG    RGB(160, 200, 255)
#define C_SEP         RGB(60,  60,  80)
#define C_HINT        RGB(90,  90, 110)

// Окно калькулятора
#define WIN_W   340
#define WIN_H   480

#define TITLE_H  28
#define DISP_H   80
#define DISP_PAD 10
#define BTN_COLS  4
#define BTN_ROWS  5
#define BTN_PAD   6

// Кнопки
typedef enum { BK_NUM, BK_OP, BK_EQ, BK_CLR, BK_FN } BtnKind;

typedef struct {
    const char* label;
    BtnKind     kind;
} Btn;

static const Btn BUTTONS[BTN_ROWS][BTN_COLS] = {
    { {"C", BK_CLR}, {"+/-", BK_FN}, {"%", BK_FN}, {"/", BK_OP} },
    { {"7", BK_NUM}, {"8",   BK_NUM}, {"9", BK_NUM}, {"*", BK_OP} },
    { {"4", BK_NUM}, {"5",   BK_NUM}, {"6", BK_NUM}, {"-", BK_OP} },
    { {"1", BK_NUM}, {"2",   BK_NUM}, {"3", BK_NUM}, {"+", BK_OP} },
    { {"0", BK_NUM}, {".",   BK_FN},  {"<",BK_FN},  {"=", BK_EQ} },
};

// Состояние калькулятора 
#define MAX_DISP 16

static char  disp[MAX_DISP + 2];
static int   dlen;
static int   has_dot;
static int   calc_state;
static long  acc;
static long  saved;
static char  op_cur;

// Эффекты кнопок (выделение, нажатие)
static int sel_r, sel_c;
static int pressed_r, pressed_c;

// Координаты окна
static int win_x, win_y;
static int btn_w, btn_h;
static int disp_y;

// Арифметика
static long do_parse(void) {
    long ip = 0, fp = 0, fdiv = 1;
    int neg = 0, i = 0, frac = 0;
    if (disp[0] == '-') { neg = 1; i = 1; }
    for (; i < dlen; i++) {
        char c = disp[i];
        if (c == '.') { frac = 1; continue; }
        if (frac) { if (fdiv < 100) { fp = fp*10 + (c-'0'); fdiv *= 10; } }
        else      { ip = ip*10 + (c-'0'); }
    }
    long v = ip*100 + (fdiv == 10 ? fp*10 : fp);
    return neg ? -v : v;
}

static void do_format(long v) {
    dlen = 0; has_dot = 0;
    int neg = (v < 0);
    long av = neg ? -v : v;
    long ip = av / 100;
    long fp = av % 100;
    if (neg) disp[dlen++] = '-';
    if (ip == 0) { disp[dlen++] = '0'; }
    else {
        char tmp[12]; int tl = 0;
        for (long t = ip; t > 0; t /= 10) tmp[tl++] = '0' + (int)(t%10);
        int i; for (i = tl-1; i >= 0; i--) disp[dlen++] = tmp[i];
    }
    if (fp) {
        disp[dlen++] = '.'; has_dot = 1;
        disp[dlen++] = '0' + (int)(fp/10);
        if (fp%10) disp[dlen++] = '0' + (int)(fp%10);
    }
    disp[dlen] = '\0';
}

static void do_reset(void) {
    disp[0]='0'; disp[1]='\0';
    dlen=1; has_dot=0;
    calc_state=0; acc=0; saved=0; op_cur=0;
}

static void do_err(void) {
    disp[0]='E'; disp[1]='r'; disp[2]='r'; disp[3]='\0';
    dlen=3; calc_state=2; op_cur=0;
}

// Отрисовка элементов

// Прямоугольник
static void rounded_rect(int x, int y, int w, int h, uint32_t c) {
    fillrect(x+1, y,   w-2, h,   c);
    fillrect(x,   y+1, w,   h-2, c);
}

// Текст по центру прямоугольника
static void centered_text(int x, int y, int w, int h, const char* s, uint32_t fg, uint32_t bg) {
    int len = slen(s);
    int tx = x + (w - len*8) / 2;
    int ty = y + (h - 8) / 2;
    if (bg != C_BG) fillrect(x, y, w, h, bg);
    int i;
    for (i = 0; s[i]; i++) gfx_char(tx + i*8, ty, s[i], fg, bg);
}

// Тень кнопки
static void draw_shadow(int x, int y, int w, int h) {
    fillrect(x+3, y+3, w, h, C_BTN_SHADOW);
}

// Кнопка
static void draw_button(int r, int c, int pressed) {
    int x = win_x + BTN_PAD + c * (btn_w + BTN_PAD);
    int y = win_y + TITLE_H + DISP_H + BTN_PAD + r * (btn_h + BTN_PAD);

    int is_sel = (r == sel_r && c == sel_c);
    BtnKind k = BUTTONS[r][c].kind;

    uint32_t bg, bg_hl, fg;
    switch (k) {
        case BK_CLR: bg=C_BTN_CLR_BG; bg_hl=C_BTN_CLR_HL; fg=C_BTN_FG; break;
        case BK_OP:  bg=C_BTN_OP_BG;  bg_hl=C_BTN_OP_HL;  fg=C_BTN_FG; break;
        case BK_EQ:  bg=C_BTN_EQ_BG;  bg_hl=C_BTN_EQ_HL;  fg=C_BTN_FG; break;
        default:     bg=C_BTN_NUM_BG; bg_hl=C_BTN_NUM_HL;  fg=C_BTN_FG; break;
    }

    uint32_t use_bg = pressed ? C_BTN_NUM_PR : (is_sel ? bg_hl : bg);

    if (!pressed) draw_shadow(x, y, btn_w, btn_h);
    rounded_rect(x, y, btn_w, btn_h, use_bg);

    // блик
    if (!pressed)
        fillrect(x+1, y+1, btn_w-2, 2, RGB(255,255,255));

    centered_text(x, y, btn_w, btn_h, BUTTONS[r][c].label, fg, use_bg);

    // Рамка
    if (is_sel && !pressed)
        rect(x, y, btn_w, btn_h, RGB(255,255,150));
}

// Отображение
static void draw_display(void) {
    int dx = win_x + DISP_PAD;
    int dy = win_y + TITLE_H + DISP_PAD;
    int dw = WIN_W - DISP_PAD*2;
    int dh = DISP_H - DISP_PAD*2;

    // Фон
    rounded_rect(dx, dy, dw, dh, C_DISP_BG);
    rect(dx, dy, dw, dh, C_WIN_BORDER);

    // Текущая операция - маленький текст слева
    if (op_cur && calc_state >= 1) {
        char op_s[2] = { op_cur, '\0' };
        gfx_char(dx+6, dy+6, op_s[0], C_DISP_SMALL, C_DISP_BG);
    }

    // Значение
    int len = slen(disp);
    int char_w = (dlen > 10) ? 12 : 20; // если число длинное = уменьшаем
    int text_w = len * char_w;
    int tx = dx + dw - text_w - 8;
    int ty = dy + dh/2 - 10;
    if (tx < dx+4) tx = dx+4;

    // Рисуем цифры - один символ 8px, шаг зависит от char_w
    int i;
    for (i = 0; disp[i]; i++)
        gfx_char(tx + i*char_w, ty + (char_w == 20 ? 4 : 8), disp[i], C_DISP_FG, C_DISP_BG);
}

// Заголовок окна
static void draw_titlebar(void) {
    fillrect(win_x, win_y, WIN_W, TITLE_H, C_WIN_BORDER);

    // Кнопка закрытия
    int bx = win_x + WIN_W - 22;
    int by = win_y + 6;
    fillrect(bx, by, 16, 16, C_BTN_CLR_BG);
    gfx_char(bx+4, by+4, 'X', C_BTN_FG, C_BTN_CLR_BG);

    // Заголовок
    int i;
    const char* title = " ZALCULATOR ";
    int tlen = slen(title);
    int tx = win_x + (WIN_W - tlen*8)/2;
    for (i = 0; title[i]; i++)
        gfx_char(tx + i*8, win_y + 10, title[i], C_TITLE_FG, C_WIN_BORDER);
}

// Полный перерисов окна
static void draw_all(void) {
    // Фон
    fillrect(0, 0, screen_w(), screen_h(), C_BG);

    // Тень окна
    fillrect(win_x+6, win_y+6, WIN_W, WIN_H, C_BTN_SHADOW);

    // Тело окна
    rounded_rect(win_x, win_y, WIN_W, WIN_H, C_WIN_BG);
    rect(win_x, win_y, WIN_W, WIN_H, C_WIN_BORDER);

    draw_titlebar();

    // Разделитель
    fillrect(win_x + BTN_PAD, win_y + TITLE_H + DISP_H - 2,
             WIN_W - BTN_PAD*2, 1, C_SEP);

    draw_display();

    int r, c;
    for (r = 0; r < BTN_ROWS; r++)
        for (c = 0; c < BTN_COLS; c++)
            draw_button(r, c, 0);

    // Подсказка
    const char* hint = "arrows+Enter / keyboard / ESC";
    int hlen = slen(hint);
    int hx = win_x + (WIN_W - hlen*8)/2;
    int hy = win_y + WIN_H - 14;
    int i;
    for (i = 0; hint[i]; i++)
        gfx_char(hx + i*8, hy, hint[i], C_HINT, C_WIN_BG);
}

// Перерисов только кнопок
static void redraw_btns(void) {
    int r, c;
    for (r = 0; r < BTN_ROWS; r++)
        for (c = 0; c < BTN_COLS; c++)
            draw_button(r, c, 0);
}

// Логика кнопок
static void calc_press(int r, int c) {
    const char* lbl = BUTTONS[r][c].label;
    char ch = lbl[0];

    if (ch == 'C') { do_reset(); return; }

    if (ch >= '0' && ch <= '9') {
        if (calc_state == 1) { disp[0]='0'; disp[1]='\0'; dlen=1; has_dot=0; calc_state=2; dlen=0; }
        else if (calc_state == 2 && op_cur == 0) { do_reset(); dlen=0; }
        if (dlen == 1 && disp[0]=='0') dlen=0;
        if (dlen < MAX_DISP) { disp[dlen++]=ch; disp[dlen]='\0'; }
        if (calc_state == 1) calc_state = 2;
        acc = do_parse();
        return;
    }

    if (ch == '.') {
        if (calc_state == 1)  { dlen=0; disp[0]='\0'; calc_state=2; }
        else if (calc_state == 2 && op_cur == 0) { do_reset(); dlen=0; }
        if (!has_dot) {
            if (dlen==0) { disp[dlen++]='0'; }
            disp[dlen++]='.'; disp[dlen]='\0'; has_dot=1;
        }
        acc = do_parse();
        return;
    }

    if (ch == '<') {
        if (calc_state == 2 && op_cur == 0) { do_reset(); return; }
        if (dlen > 1) {
            if (disp[dlen-1]=='.') has_dot=0;
            disp[--dlen]='\0';
        } else { disp[0]='0'; disp[1]='\0'; dlen=1; has_dot=0; }
        acc = do_parse();
        return;
    }

    if (ch == '+' && lbl[1] == '/') {
        if (disp[0] == '-') {
            int i; for (i=0; i<dlen; i++) disp[i]=disp[i+1]; dlen--;
        } else if (dlen < MAX_DISP) {
            int i; for (i=dlen; i>=0; i--) disp[i+1]=disp[i];
            disp[0]='-'; dlen++;
        }
        acc = do_parse();
        return;
    }

    if (ch == '%') {
        acc = do_parse() / 100;
        do_format(acc);
        calc_state=2; op_cur=0;
        return;
    }

    if (ch=='+' || ch=='-' || ch=='*' || ch=='/') {
        if (calc_state == 1) { op_cur=ch; return; }
        if (op_cur != 0 && calc_state == 2) {
            long a=saved, b=acc;
            long res=0;
            if      (op_cur=='+') res=a+b;
            else if (op_cur=='-') res=a-b;
            else if (op_cur=='*') res=(a/100)*b + (a%100)*b/100;
            else if (op_cur=='/') { if(b==0){do_err();return;} res=(a*100)/b; }
            acc=res; do_format(res);
        } else { acc=do_parse(); }
        saved=acc; op_cur=ch; calc_state=1;
        return;
    }

    if (ch == '=') {
        if (op_cur == 0) return;
        if (calc_state == 1) acc = saved;
        long a=saved, b=acc, res=0;
        if      (op_cur=='+') res=a+b;
        else if (op_cur=='-') res=a-b;
        else if (op_cur=='*') res=(a/100)*b + (a%100)*b/100;
        else if (op_cur=='/') { if(b==0){do_err();return;} res=(a*100)/b; }
        acc=res; do_format(res);
        op_cur=0; calc_state=2;
        return;
    }
}

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;

    int sw = screen_w();
    int sh = screen_h();

    // Выставление окна в центр
    win_x = (sw - WIN_W) / 2;
    win_y = (sh - WIN_H) / 2;

    // Вычисление размера кнопок
    btn_w = (WIN_W - BTN_PAD * (BTN_COLS + 1)) / BTN_COLS;
    btn_h = (WIN_H - TITLE_H - DISP_H - BTN_PAD * (BTN_ROWS + 1) - 20) / BTN_ROWS;

    do_reset();
    sel_r = 4; sel_c = 3;
    draw_all();

    while (1) {
        char k = waitkey();
        unsigned char uk = (unsigned char)k;
        int moved = 0;

        if (uk == (unsigned char)KEY_UP)    { sel_r = sel_r > 0 ? sel_r-1 : BTN_ROWS-1; moved=1; }
        else if (uk == (unsigned char)KEY_DOWN)  { sel_r = sel_r < BTN_ROWS-1 ? sel_r+1 : 0; moved=1; }
        else if (uk == (unsigned char)KEY_LEFT)  { sel_c = sel_c > 0 ? sel_c-1 : BTN_COLS-1; moved=1; }
        else if (uk == (unsigned char)KEY_RIGHT) { sel_c = sel_c < BTN_COLS-1 ? sel_c+1 : 0; moved=1; }
        else if (k == KEY_ENTER || k == KEY_SPACE) {
            // Эффект нажатия
            draw_button(sel_r, sel_c, 1);
            delay(80);
            calc_press(sel_r, sel_c);
            draw_display();
            draw_button(sel_r, sel_c, 0);
            continue;
        }
        else if (k == KEY_ESC) break;
        else {
            // Клавиатурные шорткаты
            int r=-1, c=-1;
            if (k>='0' && k<='9') {
                int rr, cc;
                for (rr=0;rr<BTN_ROWS;rr++) for (cc=0;cc<BTN_COLS;cc++)
                    if (BUTTONS[rr][cc].label[0]==k && BUTTONS[rr][cc].label[1]=='\0')
                        { r=rr; c=cc; }
            }
            else if (k=='+') { r=3;c=3; }
            else if (k=='-') { r=2;c=3; }
            else if (k=='*') { r=1;c=3; }
            else if (k=='/') { r=0;c=3; }
            else if (k=='=') { r=4;c=3; }
            else if (k=='.') { r=4;c=1; }
            else if (k==KEY_BACKSPACE) { r=4;c=2; }
            else if (k=='c'||k=='C') { r=0;c=0; }
            if (r >= 0) {
                // Эффект нажатой кнопки
                int old_sr = sel_r, old_sc = sel_c;
                draw_button(r, c, 1);
                delay(80);
                calc_press(r, c);
                draw_display();
                sel_r = r; sel_c = c;
                draw_button(r, c, 0);
                sel_r = old_sr; sel_c = old_sc;
                draw_button(r, c, 0);
                continue;
            }
        }

        if (moved) redraw_btns();
    }

    clear();
    return 0;
}