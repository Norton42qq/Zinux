ASM     = nasm
CC      = x86_64-elf-gcc
LD      = x86_64-elf-ld
DD      = dd
QEMU    = qemu-system-x86_64

ASMFLAGS = -f bin
CFLAGS   = -ffreestanding -m64 -O2 -Wall -Wextra
LDFLAGS  = -nostdlib -static

BOOT      = boot/boot.asm

BOOT_BIN    = boot.bin
IMG         = zinux.img

SECTOR_SIZE = 512
BOOT_SECTORS = 1
KERNEL_SECTORS = 1024

all: $(IMG)

$(IMG): $(BOOT_BIN)
	@echo "Creating zinux.img..."
	@$(DD) if=/dev/zero of=$(IMG) bs=$(SECTOR_SIZE) count=2048 status=none
	@$(DD) if=$(BOOT_BIN) of=$(IMG) bs=$(SECTOR_SIZE) count=$(BOOT_SECTORS) conv=notrunc status=none
	@echo "zinux.img created!"

$(BOOT_BIN): $(BOOT)
	@echo "Compiling boot.asm..."
	@$(ASM) $(ASMFLAGS) $< -o $@

clean:
	@echo "Cleaning..."
	@rm -f $(BOOT_BIN) $(IMG)

rund: $(IMG)
	@$(QEMU) -drive format=raw,file=$(IMG) -serial stdio -m 512M -cpu qemu64,+smep -monitor vc -d int,cpu_reset,guest_errors,unimp

.PHONY: all clean rund

