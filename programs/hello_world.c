void write(int fd, const void *buf, unsigned int len) {
    asm volatile("int $0x80" :: "a"(2), "b"(fd), "c"(buf), "d"(len));
}

void _start() {
     write(1, "Hello world", 11);
     for (;;);
}
