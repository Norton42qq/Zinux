#include "vga.h"
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
    vga_set_color(VGA_COLOR_DARK_GREY, VGA_COLOR_BLACK);
    vga_puts("  [.] ");
    vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
    vga_puts(msg);
    vga_puts("...");
}

static void kok(void) {
    vga_set_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    vga_puts(" OK\n");
    vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
}

static void kskip(const char* reason) {
    vga_set_color(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    vga_puts(" SKIP");
    if (reason) { vga_puts(" ("); vga_puts(reason); vga_puts(")"); }
    vga_puts("\n");
    vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
}

void kernel_main(void) {
    vga_init();

    vga_set_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
    vga_puts("Zinux debug checking\n");
    vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);

    kstep("system");    system_init();    kok();
    kstep("keyboard");  keyboard_init();  kok();

    kstep("IRQ"); enable_interrupts(); kok();
        kstep("disk");
        ata_init();
        if (DISK_BUF_FLAG > 0) {
            vga_set_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
            vga_puts(" OK (RAM disk, ");
            vga_put_dec(DISK_BUF_FLAG / 2);
            vga_puts("KB)\n");
            vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
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
                vga_set_color(VGA_COLOR_DARK_GREY, VGA_COLOR_BLACK);
                vga_puts("  [~] Syncing init data...");
                if (ata_flush() == 0)
                    vga_puts(" OK\n");
                else
                    vga_puts(" ERR\n");
                vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
            }
        }

        vga_puts("\n");
        shell_run();

        int dirty = ata_dirty_count();
        if (dirty > 0) {
            vga_set_color(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
            vga_puts("  [~] Flushing ");
            vga_put_dec(dirty);
            vga_puts(" sectors...");
            int r = ata_flush();
            if (r == 0) {
                vga_set_color(VGA_COLOR_LIGHT_GREEN, VGA_COLOR_BLACK);
                vga_puts(" OK\n");
            } else {
                vga_set_color(VGA_COLOR_LIGHT_RED, VGA_COLOR_BLACK);
                vga_puts(" ERRORS\n");
            }
            vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
        }
    system_halt();
}