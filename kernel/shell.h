#ifndef SHELL_H
#define SHELL_H

#include "vesa.h"

#define SHELL_BUFFER_SIZE   256
#define MAX_ARGS            16

typedef struct {
    char       username[32];
    char       hostname[32];
    int        prompt_style;
    uint8_t theme_color;
} shell_config_t;

extern shell_config_t current_config;

const char* shell_get_cwd(void);

void shell_init(void);
void shell_set_fs_ready(int v);
void shell_load_config(void);
void shell_save_config(void);
void shell_run(void);
void shell_print_welcome(void);
void shell_prompt(void);
void shell_execute(const char* cmdline);

void cmd_help(int argc, char* argv[]);
void cmd_clear(int argc, char* argv[]);
void cmd_info(int argc, char* argv[]);
void cmd_echo(int argc, char* argv[]);
void cmd_reboot(int argc, char* argv[]);
void cmd_halt(int argc, char* argv[]);
void cmd_time(int argc, char* argv[]);
void cmd_date(int argc, char* argv[]);
void cmd_version(int argc, char* argv[]);
void cmd_mem(int argc, char* argv[]);
void cmd_cpu(int argc, char* argv[]);
void cmd_ls(int argc, char* argv[]);
void cmd_cat(int argc, char* argv[]);
void cmd_mkdir(int argc, char* argv[]);
void cmd_set(int argc, char* argv[]);
void cmd_cd(int argc, char* argv[]); 
void cmd_sync(int argc, char* argv[]);

#endif