# Zinux — Makefile (Rust-ядро)
# Сборка:    make
# Запуск:    make run
# Программы: make prog          — собрать .zxe
#            make prog-install  — собрать и установить в образ
#            make prog-run      — собрать, установить и запустить

RUSTUP   ?= rustup run nightly
QEMU     ?= qemu-system-i386
NASM     ?= nasm

RUST_DIR  = rust-kernel
BUILD_DIR = build
IMG       = zinux.img
DISK_SIZE = 64

# ─────────────────────────────────────────────
#  Ядро + образ
# ─────────────────────────────────────────────

.PHONY: all run clean prog prog-run prog-install

all: $(IMG)

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/stage1.bin: boot/stage1.asm | $(BUILD_DIR)
	$(NASM) -f bin $< -o $@

$(BUILD_DIR)/stage2.bin: boot/stage2.asm | $(BUILD_DIR)
	$(NASM) -f bin $< -o $@

$(RUST_DIR)/target/i686-zinux/release/zcore:
	touch $(RUST_DIR)/zcore/src/main.rs
	cd $(RUST_DIR) && $(RUSTUP) cargo rustc --package zcore --release -- \
		-C lto=fat -C codegen-units=1

$(IMG): $(BUILD_DIR)/stage1.bin $(BUILD_DIR)/stage2.bin $(RUST_DIR)/target/i686-zinux/release/zcore
	@echo "=== Создание $(IMG) ==="
	dd if=/dev/zero of=$@ bs=1M count=$(DISK_SIZE) 2>/dev/null
	dd if=$(BUILD_DIR)/stage1.bin    of=$@ conv=notrunc bs=512 seek=0  status=none
	dd if=$(BUILD_DIR)/stage2.bin    of=$@ conv=notrunc bs=512 seek=1  status=none
	dd if=$(RUST_DIR)/target/i686-zinux/release/zcore of=$@ conv=notrunc bs=512 seek=13 status=none
	mkfs.fat -F16 -n "ZINUX" -S 512 --offset 2048 $@ 2>/dev/null || true
	mmd -i $@@@1048576 ::/BIN 2>/dev/null || true
	mmd -i $@@@1048576 ::/CONF 2>/dev/null || true
	mmd -i $@@@1048576 ::/GAMES 2>/dev/null || true
	mcopy -i $@@@1048576 programs/zeofetch.zxe  ::/BIN/ZEOFETCH.zxe
	mcopy -i $@@@1048576 programs/zalc.zxe      ::/BIN/ZALC.zxe
	mcopy -i $@@@1048576 programs/zaint.zxe      ::/BIN/ZAINT.zxe
	mcopy -i $@@@1048576 programs/zguidemo.zxe   ::/BIN/ZGUIDEMO.zxe
	mcopy -i $@@@1048576 programs/zello.zxe      ::/BIN/ZELLO.zxe
	mcopy -i $@@@1048576 programs/zetris.zxe     ::/GAMES/ZETRIS.zxe
	mcopy -i $@@@1048576 programs/znake.zxe      ::/GAMES/ZNAKE.zxe
	mcopy -i $@@@1048576 programs/password.zxe   ::/BIN/PASSWORD.zxe
	@echo "=== Готово! ==="

run: all
	$(QEMU) -drive file=$(IMG),format=raw,index=0,media=disk \
		-m 64 -vga std -accel tcg -machine pc,usb=off

clean:
	rm -rf $(BUILD_DIR) $(IMG)
	cd $(RUST_DIR) && $(RUSTUP) cargo clean 2>/dev/null || true

# ─────────────────────────────────────────────
#  Пользовательские программы (.ZXE)
# ─────────────────────────────────────────────

prog:
	$(MAKE) -C rust-sdk pack

prog-install: prog
	@# Копировать свежесобранные .zxe из rust-sdk в programs/
	@for f in rust-sdk/*.zxe; do cp -v "$$f" "programs/$$(basename $$f)"; done
	$(MAKE) all

prog-run: prog
	$(MAKE) run
