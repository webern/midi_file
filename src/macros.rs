/*!
The `macros` module provides macros for internal use.
!*/

///
macro_rules! write_u8 {
    ($w:expr, $val:expr) => {
        $w.write_all(&[$val]).context(wr!())
    };
}
