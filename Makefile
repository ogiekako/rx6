OBJS = main.o \
#	bio.o\
#	console.o\
#	exec.o\
#	file.o\
#	fs.o\
#	ide.o\
#	ioapic.o\
#	kalloc.o\
#	kbd.o\
#	lapic.o\
#	log.o\
#	main.o\
#	mp.o\
#	picirq.o\
#	pipe.o\
#	proc.o\
#	sleeplock.o\
#	spinlock.o\
#	string.o\
#	swtch.o\
#	syscall.o\
#	sysfile.o\
#	sysproc.o\
#	timer.o\
#	trapasm.o\
#	trap.o\
#	uart.o\
#	vectors.o\
#	vm.o\


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
	(cd bootmain && xargo build --target $(ARCH)-unknown-linux-gnu --release)
	ld $(LDFLAGS) -N \
		-e start \
		-Ttext 0x7C00 \
		-o bootblock.o \
		bootasm.o bootmain/target/$(ARCH)-unknown-linux-gnu/release/libbootmain.a
	$(OBJDUMP) -S bootblock.o > bootblock.asm
	$(OBJCOPY) -S -O binary -j .text bootblock.o bootblock
	./sign.pl bootblock

kernel: entrypgdir.o entry.o kern kernel.ld
	$(LD) $(LDFLAGS) -T kernel.ld -o kernel entry.o entrypgdir.o kern/target/$(ARCH)-unknown-linux-gnu/release/libkern.a  -b binary
	$(OBJDUMP) -S kernel > kernel.asm
	$(OBJDUMP) -t kernel | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > kernel.sym

kern: $(wildcard kern/src/*.rs)
	(cd kern && xargo build --target $(ARCH)-unknown-linux-gnu --release)

clean:
	(cd bootmain && xargo clean)
	rm -f *.o *.d *.a rx6.img bootblock

test:
	(cd bootmain && xargo test) # TODO: test i386

