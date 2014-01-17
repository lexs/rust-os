global _start
_start:
    mov ebx, 1337 ; exit code 127
    mov eax, 1 ; syscall #1, exit
    int 0x80
