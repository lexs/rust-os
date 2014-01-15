; Common stub called by all isr routines
isr_common_stub:
    pusha                    ; Pushes edi,esi,ebp,esp,ebx,edx,ecx,eax

    mov ax, ds               ; Lower 16-bits of eax = ds.
    push eax                 ; save the data segment descriptor

    mov ax, 0x10  ; load the kernel data segment descriptor
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push esp

    extern isr_handler
    call isr_handler

    add esp, 4

    pop eax        ; reload the original data segment descriptor
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    popa                     ; Pops edi,esi,ebp...
    add esp, 8     ; Cleans up the pushed error code and pushed ISR number
    ;sti
    iret           ; pops 5 things at once: CS, EIP, EFLAGS, SS, and ESP

%macro ISR_HANDLER 1
    global _isr_handler_%1

    _isr_handler_%1:
        ;cli
        push dword 0 ; push a dummy error code
        push dword %1
        jmp isr_common_stub
%endmacro

%macro ISR_HANDLER_ERROR 1
    global _isr_handler_%1

    _isr_handler_%1:
        ;cli
        push dword %1
        jmp isr_common_stub
%endmacro

ISR_HANDLER 0
ISR_HANDLER 1
ISR_HANDLER 2
ISR_HANDLER 3
ISR_HANDLER 4
ISR_HANDLER 5
ISR_HANDLER 6
ISR_HANDLER 7
ISR_HANDLER_ERROR 8
ISR_HANDLER 9
ISR_HANDLER_ERROR 10
ISR_HANDLER_ERROR 11
ISR_HANDLER_ERROR 12
ISR_HANDLER_ERROR 13
ISR_HANDLER_ERROR 14

%assign i 15
%rep 256 - 15
    ISR_HANDLER i
%assign i i+1
%endrep

global isr_handler_array
isr_handler_array:

%macro ISR_HANDLER_ENTRY 1
    dd _isr_handler_%1
%endmacro

%assign i 0
%rep 256
    ISR_HANDLER_ENTRY i
%assign i i+1
%endrep
