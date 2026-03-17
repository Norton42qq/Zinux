#include "shell.h"
#include "vesa.h"
#include "keyboard.h"
#include "string.h"
#include "system.h"
#include "io.h"
#include "fat16.h"
#include "zxe.h"
#include "ata.h"

shell_config_t current_config;
static char current_dir[64]="/";  // корень
static int fs_ready = 0;
void shell_set_fs_ready(int v) { fs_ready = v; }

const char* shell_get_cwd(void) { return current_dir; }

// Дефолтный конфиг до ручной замены CONFIG

static void config_default(void){
    strcpy(current_config.username,"user");
    strcpy(current_config.hostname,"zinux");
    current_config.prompt_style=0;
    current_config.theme_color=COLOR_LIGHT_GREEN;
}

static void parse_kv(char* line){
    char* eq=line; 
    while(*eq&&*eq!='=') eq++;
    if(!*eq) return;
    char* val=eq+1;
    int vlen=strlen(val);
    while(vlen>0&&(val[vlen-1]=='\r'||val[vlen-1]=='\n')) val[--vlen]='\0';
    if(strncmp(line,"user=",5)==0){
        strncpy(current_config.username,val,31);
        current_config.username[31]='\0';
    } else if(strncmp(line,"host=",5)==0){
        strncpy(current_config.hostname,val,31);
        current_config.hostname[31]='\0';
    } else if(strncmp(line,"style=",6)==0){
        current_config.prompt_style=atoi(val);
    } else if(strncmp(line,"color=",6)==0){
        current_config.theme_color=(uint8_t)atoi(val);
    }
}

void shell_load_config(void){
    config_default();

    static char cbuf[256];
    int n=fat16_read_file("CONF/CONFIG",cbuf,255);
    if(n<=0) return;
    cbuf[n]='\0';
    char* line=strtok(cbuf,"\n");
    while(line){
        parse_kv(line);
        line=strtok(NULL,"\n");
    }
}

void shell_save_config(void){
    static char cbuf[256]; 
    int n=0;
    strcpy(cbuf+n,"user="); n+=5;
    strcpy(cbuf+n,current_config.username); n+=strlen(current_config.username);
    cbuf[n++]='\n';
    strcpy(cbuf+n,"host="); n+=5;
    strcpy(cbuf+n,current_config.hostname); n+=strlen(current_config.hostname);
    cbuf[n++]='\n';
    strcpy(cbuf+n,"style="); n+=6;
    itoa(current_config.prompt_style,cbuf+n,10); n+=strlen(cbuf+n);
    cbuf[n++]='\n';
    strcpy(cbuf+n,"color="); n+=6;
    itoa((int)current_config.theme_color,cbuf+n,10); n+=strlen(cbuf+n);
    cbuf[n++]='\n';
    cbuf[n]='\0';
    
    if(fat16_write_file("CONF/CONFIG",cbuf,(uint32_t)n)>0){
        vesa_set_color(COLOR_DARK_GREY,COLOR_BLACK);
        vesa_print("  [Config saved to CONF/CONFIG]\n");
        vesa_set_color(COLOR_WHITE,COLOR_BLACK);
        ata_flush();
    } else {
        vesa_set_color(COLOR_LIGHT_RED,COLOR_BLACK);
        vesa_print("  [Failed to save config]\n");
        vesa_set_color(COLOR_WHITE,COLOR_BLACK);
    }
}


// Буфер ввода
static char input_buffer[SHELL_BUFFER_SIZE];
static int input_pos = 0;

// Структура команды
typedef struct {
    const char* name;
    const char* description;
    void (*handler)(int argc, char* argv[]);
} command_t;

// Таблица команд
static const command_t commands[] = {
    { "help",    "Show available commands",     cmd_help },
    { "clear",   "Clear the screen",            cmd_clear },
    { "info",    "Show system information",     cmd_info },
    { "echo",    "Print text to screen",        cmd_echo },
    { "reboot",  "Reboot the computer",         cmd_reboot },
    { "halt",    "Halt the system",             cmd_halt },
    { "time",    "Show current time",           cmd_time },
    { "date",    "Show current date",           cmd_date },
    { "version", "Show kernel version",         cmd_version },
    { "mem",     "Show memory information",     cmd_mem },
    { "cpu",     "Show CPU information",        cmd_cpu },
    { "set",     "set user|host|style|color",   cmd_set},
    { "cd",      "cd <dir> - change directory", cmd_cd},
    { "ls",      "List files on disk",          cmd_ls },
    { "mkdir",   "mkdir <dir>",                 cmd_mkdir},
    { "cat",     "Display file contents",       cmd_cat },
    { "sync",    "Flush disk writes to media",  cmd_sync },
    { NULL, NULL, NULL }
};

// приветствие
void shell_print_welcome(void) {
    vesa_clear(COLOR_BLACK);
    vesa_set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    vesa_print("========================================\n");
    vesa_print("         Welcome to Zinux\n");
    vesa_print("    NEW RUSSIAN OPERATING SYSTEM\n");
    vesa_print("========================================\n\n");
    
    vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    vesa_print("Type 'help' for list of commands.\n\n");
     vesa_print("\x80 \x81 \x82 \x83 \x84 \x85 \x86 \x87 \x88 \x89 \x8A \x8B \x8C \x8D \x8E \x8F \x90 \x91 \x92 \x93 \x94 \x95 \x96 \x97 \x98 \x99 \x9A \x9B \x9C \x9D \x9E \x9F\n\xA0 \xA1 \xA2 \xA3 \xA4 \xA5 \xA6 \xA7 \xA8 \xA9 \xAA \xAB \xAC \xAD \xAE \xAF \xE0 \xE1 \xE2 \xE3 \xE4 \xE5 \xE6 \xE7 \xE8 \xE9 \xEA \xEB \xEC \xED \xEE \xEF\n");
   
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
}

// стили
void shell_prompt(void) {
    vesa_set_color(current_config.theme_color, COLOR_BLACK);
    
    switch(current_config.prompt_style) {
        case 1: // user@host >
            vesa_print(current_config.username);
            vesa_print_char('@');
            vesa_print(current_config.hostname);
            vesa_print(" > ");
            break;
        case 2: // user#
            vesa_print(current_config.username);
            vesa_print("\xC8");
            break;
        default: // Стандарт: user@host:~$
            vesa_print(current_config.username);
            vesa_print_char('@');
            vesa_print(current_config.hostname);
            vesa_set_color(COLOR_WHITE, COLOR_BLACK);
            vesa_print(":~$ ");
            break;
    }
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
}

// Парс оболочки
static int parse_command(char* cl,char* argv[]){
    int argc=0; 
    char* t=strtok(cl," \t");
    while(t&&argc<MAX_ARGS-1){
        argv[argc++]=t;
        t=strtok(NULL," \t");
    }
    argv[argc]=NULL; 
    return argc;
}

static void read_line(void){
    input_pos=0; 
    memset(input_buffer,0,SHELL_BUFFER_SIZE);
    while(1){
        char c=keyboard_wait_char();
        if(c=='\n'){
            vesa_print_char('\n');
            input_buffer[input_pos]='\0';
            return;
        }
        if(c=='\b'){
            if(input_pos>0){
                input_pos--;
                vesa_backspace();
            }
            continue;
        }
        if(input_pos<SHELL_BUFFER_SIZE-1){
            input_buffer[input_pos++]=c;
            vesa_print_char(c);
        }
    }
}

static int try_run(const char* name,int argc,char* argv[]){
    char p[64];
    if(strchr(name,'/')){
        return zxe_run(name,argc,argv);
    }
    
    int r=zxe_run(name,argc,argv);
    if(r!=ZXE_NOT_FOUND) return r;
    
    if(strcmp(current_dir,"/")!=0){
        strcpy(p,current_dir+1);
        strcat(p,"/");
        strcat(p,name);
        r=zxe_run(p,argc,argv);
        if(r!=ZXE_NOT_FOUND) return r;
    }
    
    strcpy(p,"BIN/");
    strcat(p,name);
    r=zxe_run(p,argc,argv);
    if(r!=ZXE_NOT_FOUND) return r;
    
    strcpy(p,"BIN/");
    strcat(p,name);
    strcat(p,".ZXE");
    return zxe_run(p,argc,argv);
}

// Выполнение команды
void shell_execute(const char* cmdline) {
    char buffer[SHELL_BUFFER_SIZE];
    char* argv[MAX_ARGS];

    strncpy(buffer, cmdline, SHELL_BUFFER_SIZE - 1);
    buffer[SHELL_BUFFER_SIZE - 1] = '\0';
    str_trim(buffer);
    
    if (buffer[0] == '\0') {
        return;
    }
    
    int argc = parse_command(buffer, argv);
    if (argc == 0) {
        return;
    }
    
    // Поиск встроенной команды
    for (int i = 0; commands[i].name != NULL; i++) {
        if (strcmp(argv[0], commands[i].name) == 0) {
            commands[i].handler(argc, argv);
            return;
        }
    }
    
    // Попытка запустить как .zxe
    int result = try_run(argv[0], argc - 1, &argv[1]);
    if (result == ZXE_OK) {
        return;
    }
    
    // Команда не найдена
    vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
    vesa_print("Command not found: ");
    vesa_print(argv[0]);
    vesa_print("\n");
    vesa_print("Type 'help' or 'ls' for available commands.\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
}

// Инициализация
void shell_init(void) {
    input_pos = 0;
    memset(input_buffer, 0, SHELL_BUFFER_SIZE);
    shell_load_config();
}

void shell_run(void) {
    shell_print_welcome();
    
    while (1) {
        shell_prompt();
        read_line();
        shell_execute(input_buffer);
    }
}

void cmd_cd(int argc, char* argv[]) {
    // Без аргументов показывает текущий путь
    if (argc < 2) {
        vesa_print(current_dir);
        vesa_print("\n");
        return;
    }

    const char* target = argv[1];

    // cd / = корень
    if (strcmp(target, "/") == 0) {
        strcpy(current_dir, "/");
        return;
    }

    // cd .. = на уровень выше
    if (strcmp(target, "..") == 0) {
        if (strcmp(current_dir, "/") != 0) {
            int len = strlen(current_dir);
            while (len > 0 && current_dir[len-1] != '/') len--;
            if (len > 1) len--;
            current_dir[len] = '\0';
            if (len == 0) strcpy(current_dir, "/");
        }
        return;
    }

    // полный путь для проверки
    char newpath[64];
    char dirpart[32];

    if (target[0] == '/') {
        strcpy(newpath, target);
        strcpy(dirpart, target + 1);
    } else {
        if (strcmp(current_dir, "/") == 0) {
            newpath[0] = '/'; newpath[1] = '\0';
            strcat(newpath, target);
            strcpy(dirpart, target);
        } else {
            vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
            vesa_print("cd: cannot go deeper than one level\n");
            vesa_set_color(COLOR_WHITE, COLOR_BLACK);
            return;
        }
    }

    int plen = strlen(dirpart);
    if (plen > 1 && dirpart[plen-1] == '/') dirpart[--plen] = '\0';

    if (dirpart[0] == '\0') {
        strcpy(current_dir, "/");
        return;
    }

    fat16_dir_entry_t entry;
    int found = 0;

    if (fat16_find_file(dirpart, &entry) == 0 && (entry.attributes & 0x10)) {
        found = 1;
    }

    if (found) {
        strcpy(current_dir, newpath);
        int l = strlen(current_dir);
        if (l > 1 && current_dir[l-1] == '/') current_dir[l-1] = '\0';
    } else {
        vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
        vesa_print("cd: ");
        vesa_print(target);
        vesa_print(": No such directory\n");
        vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    }
}

void cmd_set(int argc,char* argv[]){
    if(argc<3){
        vesa_print("Usage: set <user|host|style|color> <value>\n"); 
        return;
    }
    if(strcmp(argv[1],"user")==0){
        strncpy(current_config.username,argv[2],31);
        current_config.username[31]='\0';
        vesa_print("Username: ");
        vesa_print(current_config.username);
        vesa_print("\n");
    } else if(strcmp(argv[1],"host")==0){
        strncpy(current_config.hostname,argv[2],31);
        current_config.hostname[31]='\0';
        vesa_print("Hostname: ");
        vesa_print(current_config.hostname);
        vesa_print("\n");
    } else if(strcmp(argv[1],"style")==0){
        current_config.prompt_style=atoi(argv[2]);
        vesa_print("Style: ");
        vesa_put_dec(current_config.prompt_style);
        vesa_print("\n");
    } else if(strcmp(argv[1],"color")==0){
        current_config.theme_color=(uint8_t)atoi(argv[2]);
        vesa_print("Color: ");
        vesa_put_dec((uint32_t)current_config.theme_color);
        vesa_print("\n");
    } else {
        vesa_print("Unknown: ");
        vesa_print(argv[1]);
        vesa_print("\n");
        return;
    }
    shell_save_config();
}

void cmd_mkdir(int argc,char* argv[]){
    if (!fs_ready) { vesa_print("No filesystem mounted.\n"); return; }
    if(argc<2){
        vesa_print("Usage: mkdir <dir>\n");
        return;
    }
    if(fat16_mkdir(argv[1])==0){
        vesa_print("Created: /");
        vesa_print(argv[1]);
        vesa_print("\n");
        ata_flush();
    } else {
        vesa_set_color(COLOR_LIGHT_RED,COLOR_BLACK);
        vesa_print("Failed\n");
        vesa_set_color(COLOR_WHITE,COLOR_BLACK);
    }
}

void cmd_help(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print("\n=== Available Commands ===\n\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    
    for (int i = 0; commands[i].name != NULL; i++) {
        vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
        vesa_print("  ");
        vesa_print(commands[i].name);
        
        // Выравнивание
        int len = strlen(commands[i].name);
        for (int j = len; j < 12; j++) {
            vesa_print_char(' ');
        }
        
        vesa_set_color(COLOR_LIGHT_GREY, COLOR_BLACK);
        vesa_print("- ");
        vesa_print(commands[i].description);
        vesa_print("\n");
    }
    
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    vesa_print("\n");
}

void cmd_clear(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    vesa_clear(COLOR_BLACK);
}

void cmd_info(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    system_info_t* info = system_get_info();
    
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print("\n=== System Information ===\n\n");
    
    // Процессор
    vesa_set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    vesa_print("CPU:\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    vesa_print("  Vendor:   ");
    vesa_print((char*)info->cpu_vendor);
    vesa_print("\n\n");
    
    // Память
    vesa_set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
    vesa_print("Memory:\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    vesa_print("  Lower:    ");
    vesa_put_dec(info->mem_lower);
    vesa_print(" KB\n");
    vesa_print("  Upper:    ");
    vesa_put_dec(info->mem_upper);
    vesa_print(" KB\n\n");
}

void cmd_echo(int argc, char* argv[]) {
    for (int i = 1; i < argc; i++) {
        vesa_print(argv[i]);
        if (i < argc - 1) {
            vesa_print_char(' ');
        }
    }
    vesa_print("\n");
}

void cmd_reboot(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print("\nSyncing disk...");
    int dirty = ata_dirty_count();
    if (dirty > 0) {
        int r = ata_flush();
        if (r == 0) vesa_print(" OK\n");
        else        vesa_print(" ERRORS\n");
    } else {
        vesa_print(" nothing to sync\n");
    }
    vesa_print("Rebooting system...\n");
    
    for (volatile int i = 0; i < 10000000; i++);
    system_reboot();
}

void cmd_halt(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    int dirty = ata_dirty_count();
    if (dirty > 0) {
        vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
        vesa_print("\nSyncing disk...");
        int r = ata_flush();
        if (r == 0) vesa_print(" OK\n");
        else        vesa_print(" ERRORS\n");
    }
    vesa_clear(COLOR_BLACK);
    vesa_set_color(COLOR_BROWN, COLOR_BLACK);
    vesa_puts(225, 250, "It's now safe to turn off your computer", COLOR_BROWN, COLOR_BLACK);
    system_halt();
}

// Чтение времени
static uint8_t cmos_read(uint8_t reg) {
    outb(0x70, reg);
    return inb(0x71);
}

static uint8_t bcd_to_bin(uint8_t bcd) {
    return ((bcd >> 4) * 10) + (bcd & 0x0F);
}

void cmd_time(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    uint8_t second = bcd_to_bin(cmos_read(0x00));
    uint8_t minute = bcd_to_bin(cmos_read(0x02));
    uint8_t hour = bcd_to_bin(cmos_read(0x04));
    
    vesa_print("Current time: ");
    if (hour < 10) vesa_print_char('0');
    vesa_put_dec(hour);
    vesa_print_char(':');
    if (minute < 10) vesa_print_char('0');
    vesa_put_dec(minute);
    vesa_print_char(':');
    if (second < 10) vesa_print_char('0');
    vesa_put_dec(second);
    vesa_print("\n");
}

void cmd_date(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    uint8_t day = bcd_to_bin(cmos_read(0x07));
    uint8_t month = bcd_to_bin(cmos_read(0x08));
    uint8_t year = bcd_to_bin(cmos_read(0x09));
    uint8_t century = bcd_to_bin(cmos_read(0x32));
    
    if (century == 0) century = 20;
    
    vesa_print("Current date: ");
    if (day < 10) vesa_print_char('0');
    vesa_put_dec(day);
    vesa_print_char('/');
    if (month < 10) vesa_print_char('0');
    vesa_put_dec(month);
    vesa_print_char('/');
    vesa_put_dec(century);
    if (year < 10) vesa_print_char('0');
    vesa_put_dec(year);
    vesa_print("\n");
}

void cmd_version(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    vesa_print("Zinux version ??? Beta-branch\n");
}

void cmd_mem(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    system_info_t* info = system_get_info();
    
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print("\n=== Memory Information ===\n\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    
    vesa_print("Conventional memory: ");
    vesa_put_dec(info->mem_lower);
    vesa_print(" KB\n");
    
    vesa_print("Extended memory:     ");
    vesa_put_dec(info->mem_upper);
    vesa_print(" KB\n");
    
    uint32_t total = (info->mem_lower + info->mem_upper + 1024) / 1024;
    vesa_print("Total memory:        ~");
    vesa_put_dec(total);
    vesa_print(" MB\n\n");
}

void cmd_cpu(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    
    system_info_t* info = system_get_info();
    
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print("\n=== CPU Information ===\n\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    
    vesa_print("Vendor:   ");
    vesa_print((char*)info->cpu_vendor);
    vesa_print("\n");
    
    vesa_print("Features: ");
    if (info->cpu_features & (1 << 0)) vesa_print("FPU ");
    if (info->cpu_features & (1 << 4)) vesa_print("TSC ");
    if (info->cpu_features & (1 << 5)) vesa_print("MSR ");
    if (info->cpu_features & (1 << 23)) vesa_print("MMX ");
    if (info->cpu_features & (1 << 25)) vesa_print("SSE ");
    if (info->cpu_features & (1 << 26)) vesa_print("SSE2 ");
    vesa_print("\n\n");
}

void cmd_ls(int argc, char* argv[]) {
    if (!fs_ready) { vesa_print("No filesystem mounted.\n"); return; }
    (void)argc; (void)argv;
    fat16_list_dir(current_dir);
}

void cmd_cat(int argc, char* argv[]) {
    if (!fs_ready) { vesa_print("No filesystem mounted.\n"); return; }
    if (argc < 2) {
        vesa_print("Usage: cat <filename>\n");
        return;
    }
    char filepath[64];
    if (argv[1][0] == '/' || strcmp(current_dir, "/") == 0) {
        if (argv[1][0] == '/') {
            strncpy(filepath, argv[1] + 1, 63);
        } else {
            strncpy(filepath, argv[1], 63);
        }
    } else {
        strncpy(filepath, current_dir + 1, 31);
        strcat(filepath, "/");
        strcat(filepath, argv[1]);
    }
    filepath[63] = '\0';

    if (!fat16_file_exists(filepath)) {
        vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
        vesa_print("File not found: ");
        vesa_print(filepath);
        vesa_print("\n");
        vesa_set_color(COLOR_WHITE, COLOR_BLACK);
        return;
    }
    // проверка размера файла
    uint32_t size = fat16_file_size(filepath);
    if (size == 0) {
        vesa_print("File is empty\n");
        return;
    }
    
    // Ограничение размера для безопасности
    if (size > 64 * 1024) {
        size = 64 * 1024;
        vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
        vesa_print("Warning: showing first 64KB only\n");
        vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    }
    
    // буфер
    static uint8_t file_buffer[64 * 1024];
    
    // Чтение файла
    int bytes_read = fat16_read_file(filepath, file_buffer, size);
    if (bytes_read < 0) {
        vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
        vesa_print("Error reading file\n");
        vesa_set_color(COLOR_WHITE, COLOR_BLACK);
        return;
    }
    
    // Вывод содержимого
    vesa_print("\n");
    for (int i = 0; i < bytes_read; i++) {
        char c = file_buffer[i];
        if (c >= 32 || c == '\n' || c == '\r' || c == '\t') {
            vesa_print_char(c);
        }
    }
    vesa_print("\n");
}

void cmd_sync(int argc, char* argv[]) {
    (void)argc; (void)argv;
    int dirty = ata_dirty_count();
    if (dirty == 0) {
        vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
        vesa_print("Disk is clean, nothing to sync\n");
        vesa_set_color(COLOR_WHITE, COLOR_BLACK);
        return;
    }
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print("Syncing ");
    vesa_put_dec((uint32_t)dirty);
    vesa_print(" sector(s) to disk...");
    int r = ata_flush();
    if (r == 0) {
        vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
        vesa_print(" OK\n");
    } else {
        vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
        vesa_print(" FAILED (");
        vesa_put_dec((uint32_t)(-r));
        vesa_print(" errors)\n");
    }
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
}