TARGET=i386-intel-linux
CC=i386-elf-gcc
AS=i386-elf-as
LD=i386-elf-ld
NASM=nasm
RUSTC=rustc
RUSTCFLAGS := -O --cfg debug --target $(TARGET) --debuginfo 2 -L .
MKISOFS := mkisofs
CLANG=clang
CLANGFLAGS = -target $(TARGET) -O2 -ffreestanding

LCORE=libcore-c5ed6fb4-0.11.0-pre.rlib
LLIBC=liblibc-4f9a876d-0.11.0-pre.rlib
LRLIBC=librlibc-d1ece24e-0.11.0-pre.rlib
LALLOC=liballoc-1085c790-0.11.0-pre.rlib

QEMU=qemu-system-i386

SOURCES := $(foreach suffix, asm c, $(shell find rost -name '*.$(suffix)'))
SOURCES += boot.asm runtime.asm
OBJECTS := $(patsubst %.asm, %.o, $(patsubst %.c, %.o, $(SOURCES)))

RUST_SOURCES := $(shell find rost/ -name '*.rs')

.SUFFIXES: .o .c .rs .asm .bc

kernel.elf: linker.ld rost.o $(OBJECTS) core.o libc.o rlibc.o alloc.o do_nothing.embed hello_world.embed test_fork.embed
	$(LD) -T linker.ld -o $@ rost.o $(OBJECTS) core.o libc.o rlibc.o alloc.o do_nothing.embed hello_world.embed test_fork.embed

kernel.iso: kernel.elf
	$(MKISOFS) -quiet -R -b boot/grub/stage2_eltorito \
	    -no-emul-boot -boot-load-size 4 -boot-info-table -o $@ -V 'RUST-OS' \
	    ./iso kernel.elf

run: kernel.elf
	$(QEMU) -serial file:serial.log -kernel kernel.elf

runbochs: kernel.iso
	bochs -q

rost.o: $(RUST_SOURCES)
	$(RUSTC) $(RUSTCFLAGS) --crate-type staticlib -o $@ --emit=obj rost/mod.rs

main.o: arch/.* drivers/.* kernel/.* memory/.*

core.o: $(LCORE)
	ar -x $(LCORE) core.o

libc.o: $(LLIBC)
	ar -x $(LLIBC) libc.o

rlibc.o: $(LRLIBC)
	ar -x $(LRLIBC) rlibc.o

alloc.o: $(LALLOC)
	ar -x $(LALLOC) alloc.o

%.embed: %.elf
	i386-elf-objcopy -I binary -O elf32-i386 -B i386 $< $@

%.elf: programs/%.o
	$(LD) -o $@ $<

.asm.o:
	$(NASM) -f elf32 -Wall -o $@ $<

.rs.o:
	$(RUSTC) $(RUSTCFLAGS) --crate-type staticlib -o $@ --emit obj $<

.c.o:
	$(CLANG) $(CLANGFLAGS) -o $@ -c $<

clean:
	rm -f *.{o,bin,bc,elf,embed,iso} $(OBJECTS) programs/*.o
