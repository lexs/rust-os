#include "syscalls.h"

void _start() {
    unsigned pid = fork();

    if (pid == 0) {
        write(1, "Child\n", 6);
    } else {
        write(1, "Parent\n", 7);
    }

    exit(0);
}
