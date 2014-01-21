TARGET=i386-intel-linux
CC=i386-elf-gcc
AS=i386-elf-as
LD=i386-elf-ld
NASM=nasm
RUSTC=rustc
RUSTCFLAGS := -O --cfg debug --target $(TARGET) -Z no-landing-pads -Z debug-info
CLANG=clang
CLANGFLAGS = -target $(TARGET) -O2 -ffreestanding

LCORE=libcore-2e829c2f-0.0.rlib

QEMU=qemu-system-i386

SOURCES := $(foreach suffix, asm c, $(shell find rost -name '*.$(suffix)'))
SOURCES += boot.asm runtime.asm support.asm
OBJECTS := $(patsubst %.asm, %.o, $(patsubst %.c, %.o, $(SOURCES)))

RUST_SOURCES := $(shell find rost/ -name '*.rs')

.SUFFIXES: .o .c .rs .asm .bc

os.bin: linker.ld rost.o core.o $(OBJECTS) do_nothing.embed hello_world.embed
	$(LD) -T linker.ld -o $@ rost.o core.o $(OBJECTS) do_nothing.embed hello_world.embed

run: os.bin
	$(QEMU) -kernel os.bin

$(LCORE):
	$(RUSTC) $(RUSTCFLAGS) rust-core/core/lib.rs --out-dir .

rost.o: $(LCORE) $(RUST_SOURCES)
	$(RUSTC) $(RUSTCFLAGS) --lib -o $@ -c rost/mod.rs -L .

main.o: $(LCORE) arch/.* drivers/.* kernel/.* memory/.*

core.o: $(LCORE)
	ar -x $(LCORE) core.o

%.embed: %.elf
	i386-elf-objcopy -I binary -O elf32-i386 -B i386 $< $@

%.elf: programs/%.o
	$(LD) -o $@ $<

.asm.o:
	$(NASM) -f elf32 -Wall -o $@ $<

.rs.o:
	$(RUSTC) $(RUSTCFLAGS) --lib -o $@ -c $< -L .

.c.o:
	$(CLANG) $(CLANGFLAGS) -o $@ -c $<

clean:
	rm -f *.{o,bin,bc,rlib,elf,embed} $(OBJECTS) programs/*.o
