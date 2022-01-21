macro_rules! get {
    (mut $expr:expr, $index:expr $(, $($arg:tt)+)?) => {{
        let expr = $expr;
        let index = $index;

        debug_assert!(expr.get_mut(index.clone()).is_some() $(, $($arg)*)?);
        expr.get_unchecked_mut(index)
    }};
    ($expr:expr, $index:expr $(, $($arg:tt)+)?) => {{
        let expr = $expr;
        let index = $index;

        debug_assert!(expr.get(index.clone()).is_some() $(, $($arg)*)?);
        expr.get_unchecked(index)
    }};
}

macro_rules! utf8 {
    ($expr:expr $(, $($arg:tt)+)?) => {{
        use ::std::str::from_utf8;
        use ::std::str::from_utf8_unchecked;
        let expr = $expr;

        debug_assert!(from_utf8(expr).is_ok() $(, $($arg)*)?);
        from_utf8_unchecked(expr)
    }};
}

#[cfg(test)]
macro_rules! test_files {
    ($path:literal) => {
        test_files!(
            impl $path
                ARABIC  : "arabic"
                CODE    : "code"
                EMOJI   : "emoji"
                ENGLISH : "english"
                HINDI   : "hindi"
                JAPANESE: "japanese"
                KOREAN  : "korean"
                MANDARIN: "mandarin"
                RUSSIAN : "russian"
        );
    };
    (impl $path:literal $($NAME:ident: $name:literal)*) => {
        $(
            #[allow(unused)]
            const $NAME: &str = include_str!(concat!($path, "tests/", $name, ".txt"));
        )*

        #[allow(unused)]
        const FILES: &[&str] = &[ $($NAME,)* ];
    };
}
