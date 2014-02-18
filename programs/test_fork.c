#include "syscalls.h"

unsigned strlen(const char* str) {
    const char* s;
    for (s = str; *s; ++s);
    return s - str;
}

void loop(const char* message) {
    while (1) {
        write(1, message, strlen(message));
        sleep(100);
    }
}

void _start() {
    if (fork() != 0) {
        loop("Parent\n");
    } else {
        if (fork() != 0) {
            loop("Child 1\n");
        } else {
            loop("Child 2\n");
        }
    }
}
