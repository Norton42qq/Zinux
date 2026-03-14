ASM = nasm
CC  = i686-elf-gcc
LD = i686-elf-ld

CFLAGS = -m32 -std=c11 -ffreestanding -fno-stack-protector -fno-pie -Wall -Wextra -O2 -fno-builtin -march=i486 -c
LDFLAGS = -m elf_i386 -T zxe.ld --oformat binary

SOURCES = $(wildcard *.c)
PROGRAMS = $(SOURCES:.c=.zxe)

BUILD_DIR = build

all: $(BUILD_DIR) $(PROGRAMS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BUILD_DIR)/start.o: start.asm
	$(ASM) -f elf32 start.asm -o $(BUILD_DIR)/start.o

%.zxe: %.c zxe_api.h $(BUILD_DIR)/start.o
	@echo "=== Building $@ ==="
	$(CC) $(CFLAGS) $< -o $(BUILD_DIR)/$*.o
	$(LD) $(LDFLAGS) $(BUILD_DIR)/start.o $(BUILD_DIR)/$*.o -o $(BUILD_DIR)/$*.bin
	./mkzxe $(BUILD_DIR)/$*.bin $@ "$*"
	@echo "✓ $@ created"
	@echo ""

install: all
	@echo "=== Installing programs to Zinux disk ==="
	@count=0; \
	for prog in $(PROGRAMS); do \
		if [ -f $$prog ]; then \
			echo -n "Installing $$prog... "; \
			if mcopy -i ../zinux.img@@1048576 -o $$prog :: 2>/dev/null; then \
				echo "✓"; \
				count=$$((count + 1)); \
			else \
				echo "✗"; \
			fi; \
		fi; \
	done; \
	echo ""; \
	echo "Installed $$count program(s)"; \
	echo "Run 'cd .. && make run' to start Zinux"

clean:
	rm -rf $(BUILD_DIR) *.zxe
	@echo "✓ Cleaned"

.PHONY: all clean install