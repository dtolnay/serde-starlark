use insta::assert_display_snapshot;

#[test]
fn test_string_escape() {
    let strings: &[&str] = &[
        "\x07 \x08 \x0C \n \r \t \x0B \\",
        "Have you read \"To Kill a Mockingbird?\"",
        "Yes, it's a classic.",
        "\u{41}\u{414}\u{754c}\u{1f600}",
        "\0\00 \x01\x010 \x0E\x0E0 \x7F\x7F0 \u{80}",
    ];

    let starlark = serde_starlark::to_string(strings).unwrap();
    assert_display_snapshot!(starlark, @r###"
    [
        "\a \b \f \n \r \t \v \\",
        "Have you read \"To Kill a Mockingbird?\"",
        "Yes, it's a classic.",
        "AД界😀",
        "\0\x000 \1\x010 \16\x0E0 \177\1770 \u0080",
    ]
    "###);
}
