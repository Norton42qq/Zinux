#include "vesa.h"
#include "keyboard.h"
#include "system.h"
#include "shell.h"
#include "string.h"
#include "ata.h"
#include "fat16.h"
#include "zxe.h"
#include "api.h"
#include "io.h"
#include "panic.h"

#define DISK_BUF_FLAG (*(volatile uint32_t*)0x8004)

static void kstep(const char* msg) {
    vesa_set_color(COLOR_DARK_GREY, COLOR_BLACK);
    vesa_print("  [.] ");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
    vesa_print(msg);
    vesa_print("...");
}

static void kok(void) {
    vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    vesa_print(" OK\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
}

static void kskip(const char* reason) {
    vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
    vesa_print(" SKIP");
    if (reason) { vesa_print(" ("); vesa_print(reason); vesa_print(")"); }
    vesa_print("\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);
}

void kernel_main(void) {
    vesa_init();

    vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    vesa_print("Zinux debug checking\n");
    vesa_set_color(COLOR_WHITE, COLOR_BLACK);

    kstep("system");    system_init();    kok();
    kstep("keyboard");  keyboard_init();  kok();

    kstep("IRQ"); enable_interrupts(); kok();
        kstep("disk");
        ata_init();
        if (DISK_BUF_FLAG > 0) {
            vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
            vesa_print(" OK (RAM disk, ");
            vesa_put_dec(DISK_BUF_FLAG / 2);
            vesa_print("KB)\n");
            vesa_set_color(COLOR_WHITE, COLOR_BLACK);
        } else {
            kskip("no BIOS preload, trying ATA PIO");
        }

        kstep("FAT16");
        if (fat16_init(2048) == 0) {
            shell_set_fs_ready(1);
            kok();
            shell_load_config();
        } else {
            kskip("no disk");
        }

        kstep("API");   api_setup_table(); kok();
        kstep("ZXE");   zxe_init();        kok();
        kstep("shell"); shell_init();       kok();

        {
            int dirty = ata_dirty_count();
            if (dirty > 0) {
                vesa_set_color(COLOR_DARK_GREY, COLOR_BLACK);
                vesa_print("  [~] Syncing init data...");
                if (ata_flush() == 0)
                    vesa_print(" OK\n");
                else
                    vesa_print(" ERR\n");
                vesa_set_color(COLOR_WHITE, COLOR_BLACK);
            }
        }

        vesa_print("\n");
        shell_run();

        int dirty = ata_dirty_count();
        if (dirty > 0) {
            vesa_set_color(COLOR_YELLOW, COLOR_BLACK);
            vesa_print("  [~] Flushing ");
            vesa_put_dec(dirty);
            vesa_print(" sectors...");
            int r = ata_flush();
            if (r == 0) {
                vesa_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
                vesa_print(" OK\n");
            } else {
                vesa_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
                vesa_print(" ERRORS\n");
            }
            vesa_set_color(COLOR_WHITE, COLOR_BLACK);
        }
    system_halt();
}