use expect_test::expect;
use serde_derive::Serialize;
use serde_starlark::FunctionCall;

#[test]
#[allow(clippy::octal_escapes)]
fn test_string_escape() {
    let strings: &[&str] = &[
        "\x07 \x08 \x0C \n \r \t \x0B \\",
        "Have you read \"To Kill a Mockingbird?\"",
        "Yes, it's a classic.",
        "\u{41}\u{414}\u{754c}\u{1f600}",
        "\0\00 \x01\x010 \x0E\x0E0 \x7F\x7F0 \u{80}",
    ];

    let starlark = serde_starlark::to_string(strings).unwrap();
    let expected = expect![[r#"
        [
            "\a \b \f \n \r \t \v \\",
            "Have you read \"To Kill a Mockingbird?\"",
            "Yes, it's a classic.",
            "AД界😀",
            "\0\x000 \1\x010 \16\x0E0 \177\1770 \u0080",
        ]
    "#]];
    expected.assert_eq(&starlark);
}

#[test]
fn test_flatten_struct() {
    #[derive(Serialize)]
    struct RustLibrary {
        #[serde(flatten)]
        common: RustCommon,
        proc_macro: bool,
    }

    #[derive(Serialize)]
    struct RustCommon {
        name: &'static str,
    }

    let rust_library = RustLibrary {
        common: RustCommon { name: "syn" },
        proc_macro: false,
    };

    let function_call = FunctionCall::new("rust_library", &rust_library);
    let starlark = serde_starlark::to_string(&function_call).unwrap();
    let expected = expect![[r#"
        rust_library(
            name = "syn",
            proc_macro = False,
        )
    "#]];
    expected.assert_eq(&starlark);
}

#[test]
fn test_function_call_positional() {
    let function_call = FunctionCall::new("load", ["@rules_rust//rust:defs.bzl", "rust_library"]);
    let starlark = serde_starlark::to_string(&function_call).unwrap();
    let expected = expect![[r#"
        load(
            "@rules_rust//rust:defs.bzl",
            "rust_library",
        )
    "#]];
    expected.assert_eq(&starlark);
}
