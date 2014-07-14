#![macro_escape]

// These currently require a "use core;" at the top of the file because macros can't
// utilize "use" statements.
macro_rules! define_flags (
    ($name:ident { $($flag:ident = $value:expr),+ }) => {
        define_flags!($name: u32 { $($flag = $value),+ })
    };

    ($name:ident: $t:ty { $($flag:ident = $value:expr),* }) => {
        #[deriving(PartialEq, Eq, Clone)]
        #[packed]
        pub struct $name {
            flags: $t
        }

        #[allow(dead_code)]
        impl $name {
            fn from_int(value: $t) -> $name {
                $name { flags: value }
            }

            fn to_int(self) -> $t {
                self.flags
            }

            pub fn insert(&mut self, other: $name) {
                self.flags |= other.flags;
            }

            pub fn remove(&mut self, other: $name) {
                self.flags &= !other.flags;
            }
        }

        #[allow(dead_code)]
        impl core::ops::BitOr<$name, $name> for $name {
            #[inline(always)]
            fn bitor(&self, other: &$name) -> $name {
                $name { flags: self.flags | other.flags }
            }
        }

        #[allow(dead_code)]
        impl core::ops::BitAnd<$name, bool> for $name {
            #[inline(always)]
            fn bitand(&self, other: &$name) -> bool {
                self.flags & other.flags != 0
            }
        }

        #[allow(dead_code)]
        impl core::ops::Not<$name> for $name {
            #[inline(always)]
            fn not(&self) -> $name {
                $name { flags: !self.flags }
            }
        }

        $(
            #[allow(dead_code)]
            pub static $flag: $name = $name { flags: $value };
        )+
    };
)

#[cfg(debug)]
macro_rules! kassert (
    ($condition:expr) => {
        if !($condition) {
            let msg = concat!("assert failed: ", stringify!($condition), " at ", file!(), ":", line!());
            panic!(msg);
        }
    }
)

#[cfg(not(debug))]
macro_rules! kassert (
    ($condition:expr) => (())
)
