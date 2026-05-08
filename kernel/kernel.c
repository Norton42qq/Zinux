#include "drivers/video.h"
#include "drivers/keyboard.h"
#include "drivers/ata.h"
#include "drivers/mouse.h"
#include "shell/shell.h"
#include "system.h"
#include "string.h"
#include "fat16.h"
#include "zxe.h"
#include "api.h"
#include "io.h"
#include "panic.h"
#include "pit.h"
#include "../libc/memory.h"

#define DISK_BUF_FLAG  (*(volatile uint32_t*)0x8014)

static void kstep(const char* msg) {
    video_set_color(COLOR_DARK_GREY, COLOR_BLACK);
    video_print("  [.] ");
    video_set_color(COLOR_WHITE, COLOR_BLACK);
    video_print(msg);
    video_print("...");
}

static void kok(void) {
    video_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    video_print(" OK\n");
    video_set_color(COLOR_WHITE, COLOR_BLACK);
}

static void kskip(const char* reason) {
    video_set_color(COLOR_YELLOW, COLOR_BLACK);
    video_print(" SKIP");
    if (reason) { video_print(" ("); video_print(reason); video_print(")"); }
    video_print("\n");
    video_set_color(COLOR_WHITE, COLOR_BLACK);
}

void kernel_main(void) {
     int ret = video_init();
    if (ret != 0) {
        // Вывод напрямую через bios vga
        volatile char* vga = (volatile char*)0xB8000;
        const char* msg = "VIDEO INIT FAILED";
        for (int i = 0; msg[i]; i++) {
            vga[i*2] = msg[i];
            vga[i*2+1] = 0x4F; // белый на красном
        }
        for(;;) __asm__("hlt");
    }

    video_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
    video_print("Zinux debug checking\n");
    video_set_color(COLOR_WHITE, COLOR_BLACK);

    kstep("memory");   memory_init();    kok();
    kstep("system");    system_init();    kok();
    kstep("keyboard");  keyboard_init();  kok();
    kstep("mouse");
    mouse_init(video_width(), video_height());
    if (mouse_is_ready()) kok();
    else kskip("init failed");

    kstep("IRQ"); enable_interrupts(); kok();
    kstep("PIT"); pit_init(PIT_TARGET_HZ); kok();
        kstep("disk");
        ata_init();
        if (DISK_BUF_FLAG > 0) {
            video_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
            video_print(" OK (RAM disk, ");
            video_put_dec(DISK_BUF_FLAG / 2);
            video_print("KB)\n");
            video_set_color(COLOR_WHITE, COLOR_BLACK);
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
                video_set_color(COLOR_DARK_GREY, COLOR_BLACK);
                video_print("  [~] Syncing init data...");
                if (ata_flush() == 0)
                    video_print(" OK\n");
                else
                    video_print(" ERR\n");
                video_set_color(COLOR_WHITE, COLOR_BLACK);
            }
        }

        video_print("\n");
        shell_run();

        int dirty = ata_dirty_count();
        if (dirty > 0) {
            video_set_color(COLOR_YELLOW, COLOR_BLACK);
            video_print("  [~] Flushing ");
            video_put_dec(dirty);
            video_print(" sectors...");
            int r = ata_flush();
            if (r == 0) {
                video_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
                video_print(" OK\n");
            } else {
                video_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
                video_print(" ERRORS\n");
            }
            video_set_color(COLOR_WHITE, COLOR_BLACK);
        }
    system_halt();
}