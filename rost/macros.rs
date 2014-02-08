#[macro_escape];

// These currently require a "use core;" at the top of the file because macros can't
// utilize "use" statements.
macro_rules! define_flags (
    ($name:ident { $($flag:ident = $value:expr),+ }) => {
        define_flags!($name: u32 { $($flag = $value),+ })
    };

    ($name:ident: $t:ty { $($flag:ident = $value:expr),* }) => {
        #[packed]
        pub struct $name {
            priv flags: $t
        }

        impl $name {
            fn from_int(value: $t) -> $name {
                $name { flags: value }
            }

            fn to_int(self) -> $t {
                self.flags
            }
        }

        impl core::ops::BitOr<$name, $name> for $name {
            #[inline(always)]
            fn bitor(&self, other: &$name) -> $name {
                $name { flags: self.flags | other.flags }
            }
        }

        impl core::ops::BitAnd<$name, bool> for $name {
            #[inline(always)]
            fn bitand(&self, other: &$name) -> bool {
                self.flags & other.flags != 0
            }
        }

        impl core::ops::Not<$name> for $name {
            #[inline(always)]
            fn not(&self) -> $name {
                $name { flags: !self.flags }
            }
        }

        $(
            pub static $flag: $name = $name { flags: $value };
        )+
    };
)

#[cfg(debug)]
macro_rules! kassert (
    ($condition:expr) => {
        if !($condition) {
            use kernel::panic;
            let msg = concat!("assert failed: ", stringify!($condition), " at ", file!(), ":", line!());
            panic(msg);
        }
    }
)

#[cfg(not(debug))]
macro_rules! kassert (
    ($condition:expr) => (())
)
