OBJS = trapasm.o \
  vectors.o

# If the makefile can't find QEMU, specify its path here
# QEMU = qemu-system-i386

# Try to infer the correct QEMU
ifndef QEMU
QEMU = $(shell if which qemu > /dev/null; \
	then echo qemu; exit; \
	elif which qemu-system-i386 > /dev/null; \
	then echo qemu-system-i386; exit; \
	elif which qemu-system-x86_64 > /dev/null; \
	then echo qemu-system-x86_64; exit; \
	else \
	qemu=/Applications/Q.app/Contents/MacOS/i386-softmmu.app/Contents/MacOS/i386-softmmu; \
	if test -x $$qemu; then echo $$qemu; exit; fi; fi; \
	echo "***" 1>&2; \
	echo "*** Error: Couldn't find a working QEMU executable." 1>&2; \
	echo "*** Is the directory containing the qemu binary in your PATH" 1>&2; \
	echo "*** or have you tried setting the QEMU variable in Makefile?" 1>&2; \
	echo "***" 1>&2; exit 1)
endif

TOOLPREFIX =

ifndef TOOLPREFIX
TOOLPREFIX := $(shell if i386-elf-objdump -i 2>&1 | grep '^elf32-i386$$' > /dev/null 2>&1; \
	then echo 'i386-elf-'; \
	elif objdump -i 2>&1 | grep 'elf32-i386' > /dev/null 2>&1; \
	then echo ''; \
	else echo "*** Error: Couldn't find an i386 version of GCC/binutils." 1>&2; exit 1; fi)
endif

CC = $(TOOLPREFIX)gcc
AS = $(TOOLPREFIX)gas
LD = $(TOOLPREFIX)ld
OBJCOPY = $(TOOLPREFIX)objcopy
OBJDUMP = $(TOOLPREFIX)objdump

ARCH=i686

CFLAGS = -fno-pic -static -fno-builtin -fno-strict-aliasing -O2 -Wall -MD -ggdb -m32 -Werror -fno-omit-frame-pointer
CFLAGS += $(shell $(CC) -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)
ASFLAGS = -m32 -gdwarf-2 -Wa,-divide
LDFLAGS += -m $(shell $(LD) -V | grep elf_i386 2>/dev/null | head -n 1)

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
	$(CC) $(CFLAGS) -fno-pic -nostdinc -I. -c bootasm.S
	(cd bootmain && cross build --target $(TARGET) --release)
	$(LD) $(LDFLAGS) -N \
		-e start \
		-Ttext 0x7C00 \
		-o bootblock.o \
		bootasm.o bootmain/target/$(TARGET)/release/libbootmain.a
	$(OBJDUMP) -S bootblock.o > bootblock.asm
	$(OBJCOPY) -S -O binary -j .text bootblock.o bootblock
	./sign.pl bootblock

ifndef RELEASE
RELEASE = debug
endif

ifeq ($(RELEASE), debug)
RELEASEFLAG =
else
RELEASEFLAG = --$(RELEASE)
endif
TARGET = $(ARCH)-unknown-linux-gnu
KERN = kern/target/$(TARGET)/$(RELEASE)/libkern.a

kernel: entry.o entrypgdir.o $(KERN) $(OBJS) kernel.ld
	$(LD) $(LDFLAGS) -T kernel.ld -o kernel entry.o entrypgdir.o $(KERN) $(OBJS) -b binary 2>&1 | tee /tmp/rx6-ld.log
	if grep "warning" /tmp/rx6-ld.log > /dev/null; then rm kernel; exit 1; fi
	# -S: Display source code intermixed with disassembly. Implies -d (= --disassemble).
	$(OBJDUMP) -S kernel > kernel.asm
	# -t: Print the symbol table entries of the file.
	$(OBJDUMP) -t kernel | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > kernel.sym

kern: $(KERN)

$(KERN): $(wildcard kern/src/*.rs)
	(cd kern && cargo xbuild --target $(TARGET) $(RELEASEFLAG) --verbose)

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
