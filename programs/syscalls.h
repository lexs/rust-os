void exit(int code) {
    asm volatile("int $0x80" :: "a"(1), "b"(code));
}

void write(int fd, const void *buf, unsigned int len) {
    asm volatile("int $0x80" :: "a"(2), "b"(fd), "c"(buf), "d"(len));
}

unsigned fork() {
    unsigned value;
    asm volatile("int $0x80" : "=a"(value) : "a"(3));
    return value;
}

void sleep(int duration) {
    asm volatile("int $0x80" :: "a"(4), "b"(duration));
}
