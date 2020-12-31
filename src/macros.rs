// hmm, i don't know, i don't like.
// macro_rules! repr_enum {
//     (
//      #[repr($repr:ident)]
//      $(#$attrs:tt)* $vis:vis enum $name:ident {
//         $($(#$enum_attrs:tt)* $enum:ident = $constant:expr),* $(,)?
//      }
//     ) => {
//         #[repr($repr)]
//         $(#$attrs)*
//         $vis enum $name {
//             $($(#$enum_attrs)* $enum = $constant),*
//         }
//
//         impl ::core::convert::TryFrom<$repr> for $name {
//             type Error = $repr;
//
//             fn try_from(value: $repr) -> ::core::result::Result<Self, $repr> {
//                 $(if $constant == value { return Ok($name :: $enum); } )*
//                 Err(value)
//             }
//         }
//
//         impl ::core::convert::From<$name> for $repr {
//             fn from(value: $name) -> $repr {
//                 match value {
//                     $($name :: $enum => $constant,)*
//                 }
//             }
//         }
//     }
// }
//
// #[cfg(test)]
// mod macro_tests {
//     use std::convert::TryInto;
//     repr_enum!(
//         #[repr(u16)]
//         #[derive(Debug, Copy, Clone, Eq, PartialEq)]
//         pub enum Foo {
//             Bar = 23,
//             Baz = 101,
//         }
//     );
//     #[test]
//     fn foo() {
//         let foobar: Foo = 23.try_into().unwrap();
//         assert_eq!(Foo::Bar, foobar);
//     }
// }
