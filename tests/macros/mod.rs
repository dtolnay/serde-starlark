macro_rules! assert_snapshot {
    ($expr:expr, @$snapshot:literal) => {
        if !cfg!(miri) {
            insta::assert_display_snapshot!($expr, @$snapshot);
        }
    };
}
