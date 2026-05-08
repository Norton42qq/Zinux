ASM      = nasm
CC       = i686-elf-gcc
LD       = i686-elf-ld

ASMFLAGS = -f elf32
CFLAGS   = -m32 -std=c11 -ffreestanding -fno-stack-protector -fno-pie -Wall -Wextra -O2 -march=i486 -c
LDFLAGS  = -m elf_i386 -T linker.ld --oformat binary

BOOT_DIR   = boot
KERNEL_DIR = kernel
DRIVER_DIR = kernel/drivers
SHELL_DIR  = kernel/shell
LIBC_DIR   = libc
BUILD_DIR  = build

LIBC_OBJ = \
    $(BUILD_DIR)/libc_string.o  \
    $(BUILD_DIR)/libc_memory.o  \
    $(BUILD_DIR)/libc_stdio.o   \
    $(BUILD_DIR)/libc_stdlib.o  \
    $(BUILD_DIR)/libc_math.o    \
    $(BUILD_DIR)/libc_assert.o

KERNEL_OBJ = \
    $(BUILD_DIR)/kernel_entry.o\
    $(BUILD_DIR)/kernel.o      \
    $(BUILD_DIR)/video.o       \
    $(BUILD_DIR)/keyboard.o    \
    $(BUILD_DIR)/mouse.o       \
    $(BUILD_DIR)/panic.o       \
    $(BUILD_DIR)/system.o      \
	$(BUILD_DIR)/pit.o         \
    $(BUILD_DIR)/shell.o       \
    $(BUILD_DIR)/bios_disk.o   \
    $(BUILD_DIR)/fat16.o       \
    $(BUILD_DIR)/zxe.o         \
    $(BUILD_DIR)/api.o         \
    $(LIBC_OBJ)

all: $(BUILD_DIR) zinux.img

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/stage1.bin: $(BOOT_DIR)/stage1.asm
	$(ASM) -f bin $< -o $@

$(BUILD_DIR)/stage2.bin: $(BOOT_DIR)/stage2.asm
	$(ASM) -f bin $< -o $@

$(BUILD_DIR)/kernel_entry.o: $(KERNEL_DIR)/kernel_entry.asm
	$(ASM) $(ASMFLAGS) $< -o $@

$(BUILD_DIR)/%.o: $(KERNEL_DIR)/%.c
	$(CC) $(CFLAGS) -I$(LIBC_DIR) $< -o $@

$(BUILD_DIR)/%.o: $(DRIVER_DIR)/%.c
	$(CC) $(CFLAGS) -I$(LIBC_DIR) $< -o $@

$(BUILD_DIR)/%.o: $(SHELL_DIR)/%.c
	$(CC) $(CFLAGS) -I$(LIBC_DIR) $< -o $@

$(BUILD_DIR)/libc_%.o: $(LIBC_DIR)/%.c
	$(CC) $(CFLAGS) -I$(LIBC_DIR) $< -o $@

$(BUILD_DIR)/kernel.bin: $(KERNEL_OBJ) linker.ld
	$(LD) $(LDFLAGS) $(KERNEL_OBJ) -o $@

zinux.img: $(BUILD_DIR)/stage1.bin $(BUILD_DIR)/stage2.bin $(BUILD_DIR)/kernel.bin
	@echo "=== Creating Zinux ==="
	dd if=/dev/zero of=$@ bs=1M count=64 2>/dev/null
	dd if=$(BUILD_DIR)/stage1.bin of=$@ conv=notrunc bs=512 seek=0 2>/dev/null
	dd if=$(BUILD_DIR)/stage2.bin of=$@ conv=notrunc bs=512 seek=1 2>/dev/null
	dd if=$(BUILD_DIR)/kernel.bin of=$@ conv=notrunc bs=512 seek=9 2>/dev/null
	mkfs.fat -F16 -n "ZINUX" -S 512 --offset 2048 $@ 2>/dev/null || true

	mmd -i zinux.img@@1048576 ::/BIN
	mmd -i zinux.img@@1048576 ::/CONF
	mmd -i zinux.img@@1048576 ::/GAMES

	mcopy -i zinux.img@@1048576 programs/zeofetch.zxe ::BIN/ZEOFETCH.zxe
	mcopy -i zinux.img@@1048576 programs/zalc.zxe ::BIN/ZALC.zxe
	mcopy -i zinux.img@@1048576 programs/zaint.zxe ::BIN/ZAINT.zxe
	mcopy -i zinux.img@@1048576 programs/zguidemo.zxe ::BIN/ZGUIDEMO.zxe
	mcopy -i zinux.img@@1048576 programs/zello.zxe ::BIN/ZELLO.zxe
	mcopy -i zinux.img@@1048576 programs/zetris.zxe ::GAMES/ZETRIS.zxe
	mcopy -i zinux.img@@1048576 programs/znake.zxe ::GAMES/ZNAKE.zxe

	@echo "=== Done! Run 'make run' ==="

prog-install:
	cd developmentfolder && make clean && make install && cd ..

run: zinux.img
	qemu-system-i386 -drive file=zinux.img,format=raw,index=0,media=disk -m 64 -vga std -accel tcg \
	    -machine pc,usb=off

clean:
	rm -rf $(BUILD_DIR) zinux.img

.PHONY: all clean run prog-install