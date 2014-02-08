common_trap_handler:
    push ds
    push es
    push fs
    push gs

    pusha

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push esp

    extern trap_handler
    call trap_handler
    add esp, 4

    popa

    pop gs
    pop fs
    pop es
    pop ds

    add esp, 8 ; trap no and err
    iret

%macro TRAP_HANDLER 1
    global _trap_handler_%1

    _trap_handler_%1:
        push dword 0 ; push dummy error code
        push dword %1
        jmp common_trap_handler
%endmacro

%macro TRAP_HANDLER_ERROR 1
    global _trap_handler_%1

    _trap_handler_%1:
        push dword %1
        jmp common_trap_handler
%endmacro

TRAP_HANDLER 0
TRAP_HANDLER 1
TRAP_HANDLER 2
TRAP_HANDLER 3
TRAP_HANDLER 4
TRAP_HANDLER 5
TRAP_HANDLER 6
TRAP_HANDLER 7
TRAP_HANDLER_ERROR 8
TRAP_HANDLER 9
TRAP_HANDLER_ERROR 10
TRAP_HANDLER_ERROR 11
TRAP_HANDLER_ERROR 12
TRAP_HANDLER_ERROR 13
TRAP_HANDLER_ERROR 14

%assign i 15
%rep 256 - 15
    TRAP_HANDLER i
%assign i i+1
%endrep

global trap_handler_array
trap_handler_array:

%macro TRAP_HANDLER_ENTRY 1
    dd _trap_handler_%1
%endmacro

%assign i 0
%rep 256
    TRAP_HANDLER_ENTRY i
%assign i i+1
%endrep
