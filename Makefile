TARGET=i386-intel-linux
CC=i386-elf-gcc
AS=i386-elf-as
LD=i386-elf-ld
NASM=nasm
RUSTC=rustc
RUSTCFLAGS := -O --target $(TARGET) -Z no-landing-pads -Z debug-info

LCORE=libcore-2e829c2f-0.0.rlib

QEMU=qemu-system-i386

.SUFFIXES: .o .rs .asm .bc

os.bin: boot.o runtime.o main.o core.o handlers.o
	$(LD) -T linker.ld -o os.bin boot.o runtime.o main.o core.o handlers.o

run: os.bin
	$(QEMU) -kernel os.bin

$(LCORE):
	$(RUSTC) $(RUSTCFLAGS) rust-core/core/lib.rs --out-dir .

main.o: $(LCORE) io.rs vga.rs gdt.rs irq.rs idt.rs timer.rs keyboard.rs

core.o: $(LCORE)
	ar -x $(LCORE) core.o

.asm.o:
	$(NASM) -f elf32 -Wall -o $@ $<

.rs.o:
	$(RUSTC) $(RUSTCFLAGS) --lib -o $@ -c $< -L .

clean:
	rm *.{o,bin,bc,rlib}
