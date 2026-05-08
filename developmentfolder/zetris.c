#include "zxe_api.h"

#define BOARD_W 10
#define BOARD_H 20
#define BLOCK_SIZE 30

#define WIN_W (BOARD_W * BLOCK_SIZE + 160)
#define WIN_H (BOARD_H * BLOCK_SIZE)

#define BG_COLOR      DARK_GREY
#define BOARD_BG      BLACK
#define BORDER_COLOR  WHITE

static unsigned int rnd_seed = 1;

static void my_srand(unsigned int seed) {
    rnd_seed = seed;
}

static int my_rand(void) {
    rnd_seed = rnd_seed * 1103515245 + 12345;
    return (unsigned int)(rnd_seed / 65536) % 32768;
}

static const int tetrominos[7][4][4] = {
    {{0,0,0,0},{1,1,1,1},{0,0,0,0},{0,0,0,0}},
    {{0,0,0,0},{0,1,1,0},{0,1,1,0},{0,0,0,0}},
    {{0,0,0,0},{0,1,0,0},{1,1,1,0},{0,0,0,0}},
    {{0,0,0,0},{0,1,1,0},{1,1,0,0},{0,0,0,0}},
    {{0,0,0,0},{1,1,0,0},{0,1,1,0},{0,0,0,0}},
    {{0,0,0,0},{1,0,0,0},{1,1,1,0},{0,0,0,0}},
    {{0,0,0,0},{0,0,1,0},{1,1,1,0},{0,0,0,0}}
};

static const uint32_t colors[7] = {
    CYAN, YELLOW, MAGENTA, LIGHT_GREEN, LIGHT_RED, BROWN, LIGHT_BLUE
};

static int board[BOARD_H][BOARD_W];
static int cur_x, cur_y, cur_type, cur_rot;
static int next_type;
static int score;
static int game_over;

static void draw_block(int x, int y, uint32_t col) {
    int px = x * BLOCK_SIZE;
    int py = y * BLOCK_SIZE;
    fillrect(px, py, BLOCK_SIZE-1, BLOCK_SIZE-1, col);
    rect(px, py, BLOCK_SIZE-1, BLOCK_SIZE-1, BORDER_COLOR);
}

static void draw_board(void) {
    fillrect(0, 0, BOARD_W * BLOCK_SIZE, BOARD_H * BLOCK_SIZE, BOARD_BG);
    for (int y = 0; y < BOARD_H; y++)
        for (int x = 0; x < BOARD_W; x++)
            if (board[y][x])
                draw_block(x, y, colors[board[y][x]-1]);
}

static void get_rotated(int type, int rot, int out[4][4]) {
    const int (*src)[4] = tetrominos[type];
    for (int i = 0; i < 4; i++)
        for (int j = 0; j < 4; j++)
            out[i][j] = src[i][j];
    for (int r = 0; r < rot; r++) {
        int tmp[4][4];
        for (int i = 0; i < 4; i++)
            for (int j = 0; j < 4; j++)
                tmp[j][3-i] = out[i][j];
        for (int i = 0; i < 4; i++)
            for (int j = 0; j < 4; j++)
                out[i][j] = tmp[i][j];
    }
}

static int collides(int tx, int ty, int type, int rot) {
    int shape[4][4];
    get_rotated(type, rot, shape);
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            if (shape[y][x]) {
                int bx = tx + x;
                int by = ty + y;
                if (bx < 0 || bx >= BOARD_W || by >= BOARD_H || by < 0)
                    return 1;
                if (by >= 0 && board[by][bx])
                    return 1;
            }
        }
    }
    return 0;
}

static void merge_piece(void) {
    int shape[4][4];
    get_rotated(cur_type, cur_rot, shape);
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            if (shape[y][x]) {
                int bx = cur_x + x;
                int by = cur_y + y;
                if (by >= 0 && by < BOARD_H && bx >= 0 && bx < BOARD_W)
                    board[by][bx] = cur_type + 1;
            }
        }
    }
}

static void clear_lines(void) {
    int lines = 0;
    for (int y = BOARD_H-1; y >= 0; ) {
        int full = 1;
        for (int x = 0; x < BOARD_W; x++)
            if (!board[y][x]) { full = 0; break; }
        if (full) {
            for (int yy = y; yy > 0; yy--)
                for (int x = 0; x < BOARD_W; x++)
                    board[yy][x] = board[yy-1][x];
            for (int x = 0; x < BOARD_W; x++) board[0][x] = 0;
            lines++;
        } else {
            y--;
        }
    }
    if (lines) score += lines * 100;
}

static void spawn_piece(void) {
    cur_type = next_type;
    next_type = my_rand() % 7;
    cur_rot = 0;
    cur_x = BOARD_W/2 - 2;
    cur_y = 0;
    if (collides(cur_x, cur_y, cur_type, cur_rot))
        game_over = 1;
}

static void rotate_piece(void) {
    int new_rot = (cur_rot + 1) % 4;
    if (!collides(cur_x, cur_y, cur_type, new_rot))
        cur_rot = new_rot;
}

static void draw_current(void) {
    int shape[4][4];
    get_rotated(cur_type, cur_rot, shape);
    for (int y = 0; y < 4; y++)
        for (int x = 0; x < 4; x++)
            if (shape[y][x])
                draw_block(cur_x + x, cur_y + y, colors[cur_type]);
}

static void draw_next(void) {
    int ox = BOARD_W * BLOCK_SIZE + 20;
    int oy = 50;
    fillrect(ox, oy, 4*BLOCK_SIZE, 4*BLOCK_SIZE, BLACK);
    rect(ox, oy, 4*BLOCK_SIZE, 4*BLOCK_SIZE, WHITE);
    gfx_text(ox, oy-20, "Next:", WHITE, BG_COLOR);
    int shape[4][4];
    get_rotated(next_type, 0, shape);
    for (int y = 0; y < 4; y++)
        for (int x = 0; x < 4; x++)
            if (shape[y][x]) {
                int px = ox + x * BLOCK_SIZE;
                int py = oy + y * BLOCK_SIZE;
                fillrect(px, py, BLOCK_SIZE-1, BLOCK_SIZE-1, colors[next_type]);
                rect(px, py, BLOCK_SIZE-1, BLOCK_SIZE-1, WHITE);
            }
}

static void draw_score(void) {
    char buf[16];
    to_str(score, buf);
    gfx_text(BOARD_W * BLOCK_SIZE + 20, 200, "Score:", WHITE, BG_COLOR);
    gfx_text(BOARD_W * BLOCK_SIZE + 20, 220, buf, YELLOW, BG_COLOR);
}

static void redraw_all(void) {
    draw_board();
    draw_current();
    draw_next();
    draw_score();
}

int main(void) {
    fillrect(0, 0, WIN_W, WIN_H, BG_COLOR);
    gfx_text(10, 10, "TETRIS - Arrow keys, Space = drop, ESC = exit", WHITE, BG_COLOR);

    uint8_t h,m,s;
    get_time(&h, &m, &s);
    my_srand((h * 3600 + m * 60 + s) * 1000);

    for (int y = 0; y < BOARD_H; y++)
        for (int x = 0; x < BOARD_W; x++)
            board[y][x] = 0;
    score = 0;
    game_over = 0;
    next_type = my_rand() % 7;
    spawn_piece();

    int fall_counter = 0;
    while (!game_over) {
        redraw_all();

        if (kbhit()) {
            char key = getch();
            if (key == KEY_LEFT) {
                if (!collides(cur_x-1, cur_y, cur_type, cur_rot)) cur_x--;
            } else if (key == KEY_RIGHT) {
                if (!collides(cur_x+1, cur_y, cur_type, cur_rot)) cur_x++;
            } else if (key == KEY_DOWN) {
                if (!collides(cur_x, cur_y+1, cur_type, cur_rot)) cur_y++;
                else {
                    merge_piece();
                    clear_lines();
                    spawn_piece();
                }
            } else if (key == KEY_UP) {
                rotate_piece();
            } else if (key == KEY_SPACE) {
                while (!collides(cur_x, cur_y+1, cur_type, cur_rot)) cur_y++;
                merge_piece();
                clear_lines();
                spawn_piece();
            } else if (key == KEY_ESC) {
                break;
            }
        }

        fall_counter++;
        if (fall_counter >= 8) {
            fall_counter = 0;
            if (!collides(cur_x, cur_y+1, cur_type, cur_rot)) {
                cur_y++;
            } else {
                merge_piece();
                clear_lines();
                spawn_piece();
                if (game_over) break;
            }
        }
        delay(1000);
    }

    fillrect(0, 0, WIN_W, WIN_H, BG_COLOR);
    gfx_text(WIN_W/2-50, WIN_H/2, "Игра окончена", RED, BG_COLOR);
    char buf[32];
    to_str(score, buf);
    gfx_text(WIN_W/2-40, WIN_H/2+30, "Счет: ", WHITE, BG_COLOR);
    gfx_text(WIN_W/2+20, WIN_H/2+30, buf, YELLOW, BG_COLOR);
    gfx_text(WIN_W/2-120, WIN_H/2+60, "Нажмите любую клавишу для выхода", LIGHT_CYAN, BG_COLOR);
    waitkey();
    clear();
    return 0;
}