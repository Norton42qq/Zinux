#include "zxe_api.h"

static const char* logo[] = {
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
    0
};

#define LOGO_COLOR   WHITE
#define LABEL_COLOR  LIGHT_CYAN
#define VALUE_COLOR  WHITE
#define SEP_COLOR    DARK_GREY
#define TITLE_COLOR  LIGHT_GREEN

static void info_row(int y, const char* label, const char* value) {
    gotoxy(46, y);
    color(LABEL_COLOR, BLACK);
    print(label);
    color(SEP_COLOR, BLACK);
    print(": ");
    color(VALUE_COLOR, BLACK);
    print(value);
}

static void info_row_int(int y, const char* label, int value) {
    gotoxy(46, y);
    color(LABEL_COLOR, BLACK);
    print(label);
    color(SEP_COLOR, BLACK);
    print(": ");
    color(VALUE_COLOR, BLACK);
    print_int(value);
}

static void info_row_hex(int y, const char* label, uint32_t value) {
    gotoxy(46, y);
    color(LABEL_COLOR, BLACK);
    print(label);
    color(SEP_COLOR, BLACK);
    print(": ");
    color(VALUE_COLOR, BLACK);
    print_hex(value);
}

static void color_bar(int y) {
    gotoxy(46, y);
    int colors[] = { BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, WHITE };
    for (int i = 0; i < 8; i++) {
        color(colors[i], colors[i]);
        print("   ");
    }
    color(WHITE, BLACK);
}

int main(int argc, char* argv[]) {
    char user[32], host[32];
    get_username(user);
    get_hostname(host);
    sys_info_t info;
    sysinfo(&info);

    clear();
    int logo_y = 2;
    for (int i = 0; logo[i] != 0; i++) {
        gotoxy(1, logo_y + i);
        const char* p = logo[i];
        while (*p) {
            if (*p == '%' || *p == '@' || *p == '.' || *p == ':' || *p == '-' || *p == '+' || *p == '=' || *p == '*' || *p == '#') {
                color(WHITE, BLACK);
            }
            putchar(*p);
            p++;
        }
    }

    uint8_t h, m, s;
    uint8_t d, mo;
    uint16_t y;
    api->get_time(&h, &m, &s);
    api->get_date(&d, &mo, &y);

    gotoxy(46, 2);
    color(TITLE_COLOR, BLACK);
    print(user);
    color(SEP_COLOR, BLACK);
    print("@");
    color(TITLE_COLOR, BLACK);
    print(host);

    gotoxy(46, 3);
    color(SEP_COLOR, BLACK);
    print("---------------------");

    info_row    (4,  "OS",       info.os_name);
    info_row_int(5, "Memory", info.mem_kb); print("KB");
    info_row    (6,  "Arch",     "x32");
    info_row    (7,  "Shell",    "ZSH (Zinux Shell)");
    info_row    (8,  "Loader",   "Zrub Boot Loader");
    info_row    (9,  "FS",       "FAT16");

    gotoxy(46, 10);
    color(SEP_COLOR, BLACK);
    print("---------------------");

    gotoxy(46, 11);
    color(LABEL_COLOR, BLACK); print("Time");
    color(SEP_COLOR,   BLACK); print(": ");
    color(VALUE_COLOR, BLACK);
    if (h < 10) putchar('0'); print_int(h); putchar(':');
    if (m < 10) putchar('0'); print_int(m); putchar(':');
    if (s < 10) putchar('0'); print_int(s);

    gotoxy(46, 12);
    color(LABEL_COLOR, BLACK); print("Date");
    color(SEP_COLOR,   BLACK); print(": ");
    color(VALUE_COLOR, BLACK);
    if (d < 10) putchar('0'); print_int(d); putchar('.');
    if (mo < 10) putchar('0'); print_int(mo); putchar('.');
    print_int(y);

    gotoxy(46, 13);
    color(SEP_COLOR, BLACK);
    print("---------------------");

    gotoxy(46, 13);
    color(LABEL_COLOR, BLACK); print("Resolution");
    color(SEP_COLOR,   BLACK); print(": ");
    color(VALUE_COLOR, BLACK); print("800x600x256color");

    color_bar(15);
    println("\n\n\n\n\n\n\n\n\n\n");
    return 0;
}
