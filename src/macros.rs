//! The `macros` module provides macros for internal use.

/// A macro for conveniently writing bytes to a `Write` object and converting the error.
macro_rules! write_u8 {
    ($w:expr, $val:expr) => {
        $w.write_all(&[$val]).context(wr!())
    };
}

// TODO - maybe use const generics instead when available
/// Example: clamp!(Channel, u8, 0, 15, 0, pub);
/// Where:
/// - Channel is the name of the struct that will be created.
/// - u8 is the underlying data type
/// - 0 is the minimum allowed value (redundant in this case)
/// - 15 is the maximum allowed value
/// - 0 is the default value
/// - pub is the visibility of the struct
macro_rules! clamp {
    (
        $(#[$meta:meta])*
        $symbol:ident, $inner_type:ty, $min:expr, $max:expr, $default:expr, $visibility:vis
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
        $visibility struct $symbol($inner_type);

        impl Default for $symbol {
            fn default() -> Self {
                Self::new($default)
            }
        }

        impl $symbol {
            /// Silently clamps the value if it is out of range. See [`Self::set`].
            #[allow(dead_code)]
            $visibility const fn new(value: $inner_type) -> Self {
                let (clamped, _) = Self::clamp(value);
                Self(clamped)
            }

            /// Returns the inner value.
            #[allow(dead_code)]
            $visibility fn get(&self) -> $inner_type {
                self.0
            }

            /// Clamps and sets. Returns `true` if `value` was in range. Returns `false` if `value`
            /// was out-of-range. That is, given a valid range of `1..=5`, then `set(0)` will set
            /// the value to `1` and return `false`. `set(4)` will set the value to `4` and return
            /// `true`.
            #[allow(dead_code)]
            $visibility fn set(&mut self, value: $inner_type) -> bool {
                let (clamped, result) = Self::clamp(value);
                self.0 = clamped;
                result
            }

            /// A private const function that does the clamping.
            #[allow(unused_comparisons)]
            const fn clamp(value: $inner_type) -> ($inner_type, bool) {
                if value < $min {
                    ($min, false)
                } else if value > $max {
                    ($max, false)
                } else {
                    (value, true)
                }
            }
        }

        impl From<$inner_type> for $symbol {
            fn from(value: $inner_type) -> Self {
                Self::new(value)
            }
        }

        impl From<$symbol> for $inner_type {
            fn from(value: $symbol) -> $inner_type {
                value.0
            }
        }

        impl std::fmt::Display for $symbol {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

#[test]
#[allow(clippy::disallowed_names)]
fn clamp_test() {
    clamp!(Foo, u8, 1, 16, 1, pub);
    let foo: Foo = 0u8.into();
    let foo_val: u8 = foo.into();
    assert_eq!(1, foo_val);
    let fmted = format!("{}", Foo::new(6));
    assert_eq!("6", fmted.as_str());
}
