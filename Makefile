QEMU = qemu-system-i386

# try to generate a unique GDB port
GDBPORT = $(shell expr `id -u` % 5000 + 25000)
# QEMU's gdb stub command line changed in 0.11
QEMUGDB = $(shell if $(QEMU) -help | grep -q '^-gdb'; \
	then echo "-gdb tcp::$(GDBPORT)"; \
	else echo "-s -p $(GDBPORT)"; fi)

CFLAGS = -fno-pic -static -fno-builtin -fno-strict-aliasing -O2 -Wall -MD -ggdb -m32 -Werror -fno-omit-frame-pointer
CFLAGS += $(shell gcc -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)
LDFLAGS += -m $(shell ld -V | grep elf_i386 2>/dev/null | head -n 1)

QEMUOPTS = -drive file=rx6.img,index=0,media=disk,format=raw -smp 2 -m 512

.gdbinit: .gdbinit.tmpl
	sed "s/localhost:1234/localhost:$(GDBPORT)/" < $^ > $@

qemu-nox: rx6.img
	qemu-system-i386 -nographic $(QEMUOPTS)

qemu-nox-gdb: rx6.img .gdbinit
	@echo "*** Now run 'gdb'." 1>&2
	$(QEMU) -nographic $(QEMUOPTS) -S $(QEMUGDB)

rx6.img: bootblock
	dd if=/dev/zero of=rx6.img count=10000
	dd if=bootblock of=rx6.img conv=notrunc

bootblock: bootasm.S
	gcc $(CFLAGS) -fno-pic -nostdinc -I. -c bootasm.S
	ld $(LDFLAGS) -N \
		-e start \
		-Ttext 0x7C00 \
		-o bootblock.o \
		bootasm.o
	objdump -S bootblock.o > bootblock.ast
	objcopy -S \
		-O binary -j .text bootblock.o bootblock
	./sign.pl bootblock

