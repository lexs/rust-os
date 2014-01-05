pub fn range(start: uint, end: uint, f: |uint|) {
    let mut i = start;
    while i < end {
        f(i);
        i += 1;
    }
}
