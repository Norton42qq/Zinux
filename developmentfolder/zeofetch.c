#include "zxe_api.h"

static const char* LOGO[] = {
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

static void print_padded(int v) {
    if (v < 10) putchar('0');
    print_int(v);
}

static void separator(int x, int y) {
    gotoxy(x, y);
    color(DARK_GREY, BLACK);
    print("---------------------");
}

static void info_str(int x, int y, const char* lbl, const char* val) {
    gotoxy(x, y);
    color(LIGHT_CYAN, BLACK); print(lbl);
    color(DARK_GREY,  BLACK); print(": ");
    color(WHITE,      BLACK); print(val);
}

static void info_int(int x, int y, const char* lbl, int val, const char* sfx) {
    gotoxy(x, y);
    color(LIGHT_CYAN, BLACK); print(lbl);
    color(DARK_GREY,  BLACK); print(": ");
    color(WHITE,      BLACK); print_int(val);
    if (sfx) print(sfx);
}

static void color_bar(int x, int y) {
    gotoxy(x, y);
    uint32_t cols[8] = { RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, LIGHT_GREY, WHITE };
    int i;
    for (i = 0; i < 8; i++) { color(cols[i], cols[i]); print("   "); }
    color(WHITE, BLACK);
}

static void int_to_buf(int v, char* buf) {
    if (v == 0) { buf[0]='0'; buf[1]='\0'; return; }
    char tmp[12]; int n=0;
    while (v > 0) { tmp[n++] = '0' + v%10; v /= 10; }
    int i; for (i=0;i<n;i++) buf[i]=tmp[n-1-i];
    buf[n]='\0';
}

static void make_res(char* buf, int w, int h) {
    char tw[8], th[8];
    int_to_buf(w, tw); int_to_buf(h, th);
    int i=0, j=0;
    while (tw[j]) buf[i++]=tw[j++];
    buf[i++]='x'; j=0;
    while (th[j]) buf[i++]=th[j++];
    buf[i++]='x';
    buf[i++]='2';buf[i++]='5';buf[i++]='6';
    buf[i++]='c';buf[i++]='o';buf[i++]='l';
    buf[i++]='o';buf[i++]='r';buf[i]='\0';
}

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;
    clear();

    sys_info_t info;
    sysinfo(&info);
    
    uint8_t h, m, s, d, mo;
    uint16_t yr;
    get_time(&h, &m, &s);
    get_date(&d, &mo, &yr);

    int sw = screen_w();
    int sh = screen_h();
    int info_x = 46;

    int i;
    for (i = 0; LOGO[i] != 0; i++) {
        gotoxy(1, 2 + i);
        const char* p = LOGO[i];
        while (*p) {
            char c = *p;
            if (c=='%'||c=='@'||c=='.'||c==':'||c=='-'||c=='+'||c=='='||c=='*'||c=='#')
                color(WHITE, BLACK);
            else
                color(DARK_GREY, BLACK);
            putchar(c);
            p++;
        }
    }

    gotoxy(info_x, 2);
    color(LIGHT_GREEN, BLACK); print(info.username);
    color(DARK_GREY,   BLACK); print("@");
    color(LIGHT_GREEN, BLACK); print(info.hostname);

    separator(info_x, 3);
    info_str(info_x, 4,  "OS",         info.os_name);
    info_int(info_x, 5,  "Memory",     (int)info.mem_kb, "KB");
    info_str(info_x, 6,  "Arch",       "x32");
    info_str(info_x, 7,  "Shell",      "ZSH (Zinux Shell)");
    info_str(info_x, 8,  "Loader",     "Zrub Boot Loader");
    info_str(info_x, 9,  "FS",         "FAT16");
    separator(info_x, 10);

    gotoxy(info_x, 11);
    color(LIGHT_CYAN, BLACK); print("Time");
    color(DARK_GREY,  BLACK); print(": ");
    color(WHITE,      BLACK);
    print_padded(h); putchar(':'); print_padded(m); putchar(':'); print_padded(s);

    gotoxy(info_x, 12);
    color(LIGHT_CYAN, BLACK); print("Date");
    color(DARK_GREY,  BLACK); print(": ");
    color(WHITE,      BLACK);
    print_padded(d); putchar('.'); print_padded(mo); putchar('.'); print_int(yr);

    separator(info_x, 13);

    char res[32];
    make_res(res, sw, sh);
    info_str(info_x, 14, "Resolution", res);

    color_bar(info_x, 16);

    gotoxy(0, 2 + i + 1);
    color(WHITE, BLACK);
    return 0;
}