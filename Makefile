TARGET=i386-intel-linux
CC=i386-elf-gcc
AS=i386-elf-as
LD=i386-elf-ld
NASM=nasm
RUSTC=rustc
RUSTCFLAGS := -O --target $(TARGET) -Z no-landing-pads -Z debug-info
CLANG=clang
CLANGFLAGS = -target $(TARGET) -O2

LCORE=libcore-2e829c2f-0.0.rlib

QEMU=qemu-system-i386

.SUFFIXES: .o .c .rs .asm .bc

os.bin: linker.ld boot.o runtime.o main.o core.o handlers.o support.o
	$(LD) -T linker.ld -o os.bin boot.o runtime.o main.o core.o handlers.o support.o

run: os.bin
	$(QEMU) -kernel os.bin

$(LCORE):
	$(RUSTC) $(RUSTCFLAGS) rust-core/core/lib.rs --out-dir .

main.o: $(LCORE) kernel.rs util.rs io.rs vga.rs gdt.rs irq.rs idt.rs timer.rs keyboard.rs paging.rs console.rs

core.o: $(LCORE)
	ar -x $(LCORE) core.o

.asm.o:
	$(NASM) -f elf32 -Wall -o $@ $<

.rs.o:
	$(RUSTC) $(RUSTCFLAGS) --lib -o $@ -c $< -L .

.c.o:
	$(CLANG) $(CLANGFLAGS) -o $@ -c $<

clean:
	rm *.{o,bin,bc,rlib}
