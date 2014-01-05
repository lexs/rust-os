pub fn range(start: uint, end: uint, f: |uint|) {
    let mut i = start;
    while i < end {
        f(i);
        i += 1;
    }
}

pub fn convert(value: u32, f: |char|) {
    let mut result: [u8, ..20] = ['0' as u8, ..20];

    let mut n = value;
    if (n == 0) {
        f('0');
    } else if (n < 0) {
        n = -n;
        f('-');
    }

    let mut length = 0;
    while n > 0 {
        result[length] = '0' as u8 + (n % 10) as u8;
        n /= 10;
        length += 1;
    }

    while (length > 0) {
        f(result[length - 1] as char);
        length -= 1;
    }
}
