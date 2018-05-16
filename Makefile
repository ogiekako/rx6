OBJS = trapasm.o \
  vectors.o

QEMU = qemu-system-i386

CC = gcc
AS = gas
LD = ld
OBJCOPY = objcopy
OBJDUMP = objdump

ARCH=i686

CFLAGS = -fno-pic -static -fno-builtin -fno-strict-aliasing -O2 -Wall -MD -ggdb -m32 -Werror -fno-omit-frame-pointer
CFLAGS += $(shell gcc -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)
ASFLAGS = -m32 -gdwarf-2 -Wa,-divide
LDFLAGS += -m $(shell ld -V | grep elf_i386 2>/dev/null | head -n 1)

GDBPORT = 26001
# QEMU's gdb stub command line changed in 0.11
QEMUGDB = $(shell if $(QEMU) -help | grep -q '^-gdb'; \
	then echo "-gdb tcp::$(GDBPORT)"; \
	else echo "-s -p $(GDBPORT)"; fi)

QEMUOPTS = -drive file=fs.img,index=1,media=disk,format=raw -drive file=rx6.img,index=0,media=disk,format=raw -smp 2 -m 512

.gdbinit: .gdbinit.tmpl
	sed "s/localhost:1234/localhost:$(GDBPORT)/" < $^ > $@

qemu-nox: rx6.img fs.img
	$(QEMU) -nographic $(QEMUOPTS)

qemu-nox-gdb: rx6.img .gdbinit
	@echo "*** Now run 'gdb'." 1>&2
	$(QEMU) -nographic $(QEMUOPTS) -S $(QEMUGDB)

rx6.img: bootblock kernel
	dd if=/dev/zero of=rx6.img count=10000
	dd if=bootblock of=rx6.img conv=notrunc
	dd if=kernel of=rx6.img seek=1 conv=notrunc

bootblock: bootasm.S $(wildcard bootmain/src/*.rs)
	gcc $(CFLAGS) -fno-pic -nostdinc -I. -c bootasm.S
	(cd bootmain && cross build --target $(TARGET) --release)
	$(LD) $(LDFLAGS) -N \
		-e start \
		-Ttext 0x7C00 \
		-o bootblock.o \
		bootasm.o bootmain/target/$(TARGET)/release/libbootmain.a
	$(OBJDUMP) -S bootblock.o > bootblock.asm
	$(OBJCOPY) -S -O binary -j .text bootblock.o bootblock
	./sign.pl bootblock

RELEASE = debug
ifeq ($(RELEASE), debug)
RELEASEFLAG =
else
RELEASEFLAG = --$(RELEASE)
endif
TARGET = $(ARCH)-unknown-linux-gnu
KERN = kern/target/$(TARGET)/$(RELEASE)/libkern.a

kernel: entry.o entrypgdir.o $(KERN) $(OBJS) kernel.ld
	$(LD) $(LDFLAGS) -T kernel.ld -o kernel entry.o entrypgdir.o $(KERN) $(OBJS) -b binary
	$(OBJDUMP) -S kernel > kernel.asm
	$(OBJDUMP) -t kernel | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > kernel.sym

$(KERN): $(wildcard kern/src/*.rs)
	(cd kern && xargo build --target $(TARGET) $(RELEASEFLAG) --verbose)

vectors.S: vectors.pl
	perl vectors.pl > vectors.S

clean:
	(cd bootmain && cross clean)
	(cd kern && cross clean)
	rm -f *.o *.d *.a *.asm rx6.img bootblock kernel vectors.S

test:
	(cd kern && cross test --target $(TARGET))

fmt:
	(cd bootmain && cargo fmt)
	(cd kern && cargo fmt)
