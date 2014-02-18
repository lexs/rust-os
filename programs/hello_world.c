#include "syscalls.h"

void _start() {
     write(1, "Hello world", 11);
     for (;;);
}
