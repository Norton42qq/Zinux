# Zinux Rust SDK & программы — сборка пользовательских .zxe-программ на Rust
#
# Требуется: rustup nightly (rust-src), qemu-system-i386, mtools (mcopy).
# Образ Zinux собирается из корня репозитория (make) и по умолчанию
# находится в ../zinux.img.

RUSTUP   ?= rustup run nightly
IMG      ?= ../zinux.img
JSON     = i686-zinux.json
TOOL     = tools/zxe-pack/target/x86_64-unknown-linux-gnu/release/zxe-pack
PROGS    = zello zeofetch zaint password zalc zguidemo znake zetris
FATOFF   = 1048576

.PHONY: all packtool pack programs install run clean

all: pack

# Утилита-упаковщик ELF -> .ZXE (собирается хостовым тулчейном)
packtool:
	@if [ ! -x $(TOOL) ]; then \
		cd tools/zxe-pack && $(RUSTUP) cargo build --release --target x86_64-unknown-linux-gnu --config 'build.target="x86_64-unknown-linux-gnu"' ; \
	fi

# Сборка SDK + всех программ (no_std, build-std=core, линковка с 0x200030)
build:
	$(RUSTUP) cargo build --release -p zinux_sdk
	$(RUSTUP) cargo build --release $(foreach p,$(PROGS),-p $(p))

# Упаковка всех программ в .zxe (заголовок 48 байт + машинный код)
pack: build packtool
	@for p in $(PROGS); do \
		$(TOOL) target/i686-zinux/release/$$p $$p.zxe $$p; \
	done

# Сборка одной программы (пример: make one NAME=zello)
one: packtool
	$(RUSTUP) cargo build --release -p $(NAME)
	$(TOOL) target/i686-zinux/release/$(NAME) $(NAME).zxe $(NAME)

programs: pack

# Копирование всех .zxe в FAT-образ Zinux (FAT16, offset 2048 секторов)
install: pack
	@for p in $(PROGS); do \
		mcopy -i $(IMG)@@$(FATOFF) $$p.zxe ::/BIN/$$(echo $$p | tr a-z A-Z).zxe ; \
	done

# Запуск в QEMU
run: install
	qemu-system-i386 -drive file=$(IMG),format=raw,index=0,media=disk -m 64 -vga std -accel tcg -machine pc,usb=off

clean:
	cargo clean
	rm -f $(PROGS).zxe
