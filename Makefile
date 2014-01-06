TARGET=i386-intel-linux
CC=i386-elf-gcc
AS=i386-elf-as
LD=i386-elf-ld
NASM=nasm
RUSTC=rustc
RUSTCFLAGS := -O --target $(TARGET) -Z no-landing-pads -Z debug-info

LCORE=libcore-2e829c2f-0.0.rlib

QEMU=qemu-system-i386

.SUFFIXES: .o .rs .s .bc

os.bin: boot.o runtime.o main.o core.o handlers.o
	$(LD) -T linker.ld -o os.bin boot.o runtime.o main.o core.o handlers.o

run: os.bin
	$(QEMU) -kernel os.bin

$(LCORE):
	$(RUSTC) $(RUSTCFLAGS) rust-core/core/lib.rs --out-dir .

main.o: vga.rs gdt.rs irq.rs idt.rs timer.rs

main.rs: $(LCORE)

core.o: $(LCORE)
	ar -x $(LCORE) core.o

support.bc:
	$(RUSTC) $(RUSTCFLAGS) --lib --emit-llvm --passes inline rust-core/support.rs --out-dir .

.bc.o:
	clang -O2 -ffreestanding -target $(TARGET) -o $@ -c $<

.s.o:
	$(NASM) -f elf -o $@ $<

.rs.bc:
	$(RUSTC) $(RUSTCFLAGS) --lib --emit-llvm $< -L .

#.rs.o:
#	$(RUSTC) $(RUSTCFLAGS) --lib -o $@ -c $< -L rust-core

clean:
	rm *.{o,bin,bc,rlib}
