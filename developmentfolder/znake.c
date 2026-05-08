#include "zxe_api.h"

#define BOX_X   1
#define BOX_Y   1
#define BOX_W   54
#define BOX_H   28
#define FW      (BOX_W - 2)
#define FH      (BOX_H - 2)
#define FX      (BOX_X + 1)
#define FY      (BOX_Y + 1)

#define PANEL_X 57
#define PANEL_Y  1
#define PANEL_W 22
#define PANEL_H 28

// Символы
#define CH_HEAD '\xFE'
#define CH_BODY '\xFE'
#define CH_FOOD '@'
//----
#define MAX_SNAKE   512
#define SPEED_INIT   6
#define SPEED_MIN    2
#define SPEED_STEP   1

// Статистика
typedef struct { int x, y; } pt;

static pt  snake[MAX_SNAKE];
static int snake_len;
static int dx, dy;
static int ndx, ndy;
static pt  food;
static int score;
static int spd;
static int dead;

static void wait_speed(int spd) {
    delay((uint32_t)(40 + spd * 35));
}

// RNG
static uint32_t rng = 0xBEEFCAFE;
static uint32_t rnd(void) {
    rng = rng * 1664525u + 1013904223u;
    return rng;
}

// Draw helpers
static void pat(int x, int y, char ch, uint32_t fg, uint32_t bg) {
    gotoxy(x, y); color(fg, bg); putchar(ch);
}
static void sat(int x, int y, const char* s, uint32_t fg, uint32_t bg) {
    gotoxy(x, y); color(fg, bg); print(s);
}
static void iat(int x, int y, int v, uint32_t fg, uint32_t bg) {
    gotoxy(x, y); color(fg, bg); print_int(v); print("   ");
}

// Начальный экран
static void draw_start(void) {
    clear();
    window(15, 7, 50, 16, " ZNAKE ");
    sat(34, 10, "* ZNAKE *",    LIGHT_GREEN, BLUE);
    sat(20, 12, "Controls:",    WHITE,       BLUE);
    sat(20, 13, "  W / Up    - up",    LIGHT_GREY, BLUE);
    sat(20, 14, "  S / Down  - down",  LIGHT_GREY, BLUE);
    sat(20, 15, "  A / Left  - left",  LIGHT_GREY, BLUE);
    sat(20, 16, "  D / Right - right", LIGHT_GREY, BLUE);
    sat(20, 18, "  ESC       - quit",  LIGHT_GREY, BLUE);
    button(30, 21, "[ Press any key ]", 1);
}

// UI
static void draw_ui(void) {
    clear();
    box_titled(BOX_X, BOX_Y, BOX_W, BOX_H, "ZNAKE", LIGHT_GREEN, BLACK);
    fill(FX, FY, FW, FH, ' ', BLACK, BLACK);
    box_titled(PANEL_X, PANEL_Y, PANEL_W, PANEL_H, "INFO", LIGHT_CYAN, BLACK);

    sat(PANEL_X+2, PANEL_Y+2,  "Score:",  LIGHT_GREY, BLACK);
    sat(PANEL_X+2, PANEL_Y+4,  "Speed:",  LIGHT_GREY, BLACK);
    sat(PANEL_X+2, PANEL_Y+6,  "Length:", LIGHT_GREY, BLACK);
    hline(PANEL_X+1, PANEL_Y+8, PANEL_W-2, DARK_GREY, BLACK);
    sat(PANEL_X+2, PANEL_Y+10, "WASD/arrows", DARK_GREY, BLACK);
    sat(PANEL_X+2, PANEL_Y+11, "ESC to quit", DARK_GREY, BLACK);
    hline(PANEL_X+1, PANEL_Y+13, PANEL_W-2, DARK_GREY, BLACK);
    sat(PANEL_X+2, PANEL_Y+15, "Eat",      LIGHT_RED, BLACK);
    pat(PANEL_X+6, PANEL_Y+15, CH_FOOD,    LIGHT_RED, BLACK);
    sat(PANEL_X+8, PANEL_Y+15, "to grow!", LIGHT_RED, BLACK);
}

static void upd_panel(void) {
    iat(PANEL_X+10, PANEL_Y+2, score,      YELLOW,      BLACK);
    iat(PANEL_X+10, PANEL_Y+4,
        SPEED_INIT - spd + 1,              LIGHT_CYAN,  BLACK);
    iat(PANEL_X+10, PANEL_Y+6, snake_len,  LIGHT_GREEN, BLACK);
}

// Еда
static void place_food(void) {
    int fx, fy, ok, i;
    do {
        fx = (int)(rnd() % (uint32_t)FW);
        fy = (int)(rnd() % (uint32_t)FH);
        ok = 1;
        for (i = 0; i < snake_len; i++)
            if (snake[i].x == fx && snake[i].y == fy) { ok = 0; break; }
    } while (!ok);
    food.x = fx; food.y = fy;
    pat(FX+fx, FY+fy, CH_FOOD, LIGHT_RED, BLACK);
}

// Инициализация игры
static void game_init(void) {
    int i, cx, cy;
    score = 0; spd = SPEED_INIT; dead = 0; snake_len = 3;
    dx = 1; dy = 0; ndx = 1; ndy = 0;
    cx = FW / 2; cy = FH / 2;
    for (i = 0; i < snake_len; i++) { snake[i].x = cx - i; snake[i].y = cy; }
    draw_ui();
    upd_panel();
    for (i = snake_len-1; i >= 1; i--)
        pat(FX+snake[i].x, FY+snake[i].y, CH_BODY, GREEN,       BLACK);
    pat(FX+snake[0].x, FY+snake[0].y, CH_HEAD, LIGHT_GREEN, BLACK);
    place_food();
}

// Передвижение
static void game_step(void) {
    int nx, ny, ate, i;
    dx = ndx; dy = ndy;
    nx = snake[0].x + dx;
    ny = snake[0].y + dy;

    // Стена
    if (nx < 0 || nx >= FW || ny < 0 || ny >= FH) { dead = 1; return; }
    for (i = 0; i < snake_len - 1; i++)
        if (snake[i].x == nx && snake[i].y == ny) { dead = 1; return; }

    ate = (nx == food.x && ny == food.y);
    pt old_tail = snake[snake_len - 1];

    if (!ate)
        pat(FX+old_tail.x, FY+old_tail.y, ' ', BLACK, BLACK);

    for (i = snake_len-1; i >= 1; i--) snake[i] = snake[i-1];
    snake[0].x = nx; snake[0].y = ny;

    pat(FX+snake[1].x, FY+snake[1].y, CH_BODY, GREEN,       BLACK);
    pat(FX+snake[0].x, FY+snake[0].y, CH_HEAD, LIGHT_GREEN, BLACK);

    if (ate) {
        if (snake_len < MAX_SNAKE) {
            snake[snake_len] = old_tail;
            snake_len++;
        }
        score++;
        if (score % 5 == 0 && spd > SPEED_MIN) spd -= SPEED_STEP;
        upd_panel();
        place_food();
    }
}

// Game over
static void draw_gameover(void) {
    window(17, 10, 46, 10, " GAME OVER ");
    sat(center_x("GAME OVER"), 12, "GAME OVER", LIGHT_RED,   BLUE);
    sat(22, 14, "Score:  ",   LIGHT_GREY,  BLUE);
    gotoxy(30, 14); color(YELLOW,      BLUE); print_int(score);
    sat(22, 15, "Length: ",   LIGHT_GREY,  BLUE);
    gotoxy(30, 15); color(LIGHT_GREEN, BLUE); print_int(snake_len);
    button(22, 17, "[ R ] again",   1);
    button(38, 17, "[ ESC ] quit",  0);
}

// Управление
static int handle(char k) {
    unsigned char u = (unsigned char)k;
    if      (u == (unsigned char)KEY_UP    || k=='w' || k=='W') { if (dy !=  1) { ndx=0;  ndy=-1; } }
    else if (u == (unsigned char)KEY_DOWN  || k=='s' || k=='S') { if (dy != -1) { ndx=0;  ndy= 1; } }
    else if (u == (unsigned char)KEY_LEFT  || k=='a' || k=='A') { if (dx !=  1) { ndx=-1; ndy= 0; } }
    else if (u == (unsigned char)KEY_RIGHT || k=='d' || k=='D') { if (dx != -1) { ndx= 1; ndy= 0; } }
    else if (k == KEY_ESC) return 1;
    return 0;
}

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;

    draw_start();
    waitkey();

restart:
    game_init();

    delay(50);
    flushkeys();

    while (!dead) {
        char k = 0;
        while (kbhit()) k = getch();
        if (k && handle(k)) goto quit;

        game_step();

        if (!dead) wait_speed(spd);
    }

    draw_gameover();
    flushkeys();

    for (;;) {
        char k = waitkey();
        if (k == 'r' || k == 'R') goto restart;
        if (k == KEY_ESC)         break;
    }

quit:
    clear();
    color(LIGHT_GREEN, BLACK);
    print("Score: "); print_int(score); println("");
    return 0;
}