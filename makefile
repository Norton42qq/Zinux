ASM     = nasm
CC      = x86_64-elf-gcc
LD      = x86_64-elf-ld
DD      = dd
QEMU    = qemu-system-x86_64

ASMFLAGS = -f bin
CFLAGS   = -ffreestanding -m64 -O2 -Wall -Wextra
LDFLAGS  = -nostdlib -static

BOOT      = boot/boot.asm

BUILD_DIR   = build
BOOT_BIN    = $(BUILD_DIR)/boot.bin
IMG         = $(BUILD_DIR)/zinux.img

SECTOR_SIZE = 512
BOOT_SECTORS = 1
KERNEL_SECTORS = 1024

all: $(IMG)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BOOT_BIN): $(BOOT) | $(BUILD_DIR)
	@echo "Compiling boot.asm..."
	@$(ASM) $(ASMFLAGS) $< -o $@

$(IMG): $(BOOT_BIN)
	@echo "Creating zinux.img..."
	@$(DD) if=/dev/zero of=$(IMG) bs=$(SECTOR_SIZE) count=2048 status=none
	@$(DD) if=$(BOOT_BIN) of=$(IMG) bs=$(SECTOR_SIZE) count=$(BOOT_SECTORS) conv=notrunc status=none
	@echo "zinux.img created!"

clean:
	@echo "Cleaning..."
	@rm -rf $(BUILD_DIR)

.PHONY: all clean
