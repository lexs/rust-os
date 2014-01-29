use core::ptr;

use core2::intrinsics;
use core2::cast;

/**
 * Swap the values at two mutable locations of the same type, without
 * deinitialising or copying either one.
 */
#[inline]
pub fn swap<T>(x: &mut T, y: &mut T) {
    unsafe {
        // Give ourselves some scratch space to work with
        let mut tmp: T = intrinsics::uninit();
        let t: *mut T = &mut tmp;

        // Perform the swap, `&mut` pointers never alias
        let x_raw: *mut T = x;
        let y_raw: *mut T = y;
        ptr::copy_nonoverlapping_memory(t, x_raw as *T, 1);
        ptr::copy_nonoverlapping_memory(x, y_raw as *T, 1);
        ptr::copy_nonoverlapping_memory(y, t as *T, 1);

        // y and t now point to the same thing, but we need to completely forget `tmp`
        // because it's no longer relevant.
        cast::forget(tmp);
    }
}

/**
 * Replace the value at a mutable location with a new one, returning the old
 * value, without deinitialising or copying either one.
 */
#[inline]
pub fn replace<T>(dest: &mut T, mut src: T) -> T {
    swap(dest, &mut src);
    src
}
