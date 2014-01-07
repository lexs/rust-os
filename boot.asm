; Multiboot constants
%define MB_PAGE_ALIGN 1<<0                          ; align loaded modules on page boundaries
%define MB_MEMORY_INFO 1<<1                         ; provide memory map
%define MB_FLAGS (MB_PAGE_ALIGN | MB_MEMORY_INFO)   ; this is the Multiboot 'flag' field
%define MB_MAGIC 0x1BADB002                         ; 'magic number' lets bootloader find the header
 
; Multiboot header
section .multiboot
align 4
        dd MB_MAGIC
        dd MB_FLAGS
        dd -(MB_MAGIC + MB_FLAGS) ; multiboot checksum
        times 5 dd 0 ; memory settings: don't need because ELF
        dd 1 ; graphics mode: text
        dd 0 ; width: don't care
        dd 0 ; height: don't care
        dd 0 ; depth: especially don't care
 
; Our stack
section .bootstrap_stack
align 4
stack_bottom:
times 16384 db 0
stack_top:
 
section .text
global _start
_start:
    ; Set up our stack
    mov esp, stack_top

    ; Rust functions compare esp against [gs:0x30] as a sort of stack guard thing
    ; as long as we set [gs:0x30] to dword 0, it should be ok
    ;mov [gs:0x30], dword 0
    mov [gs:0x30], dword stack_bottom

    extern kernel_main
    call kernel_main

    ; If kernel_main ever returns (it shouldn't), make sure we freeze
    cli
    hlt
    jmp $
