use core2::intrinsics;

/**
 * Move a thing into the void
 *
 * The forget function will take ownership of the provided value but neglect
 * to run any required cleanup or memory-management operations on it. This
 * can be used for various acts of magick.
 */
#[inline]
pub unsafe fn forget<T>(thing: T) { intrinsics::forget(thing); }