serde_starlark
==============

Serde serializer for generating syntactically valid Starlark, the declarative
format used for describing build targets in build systems including [Bazel],
[Buck], [Pants], and [Please].

[Bazel]: https://bazel.build
[Buck]: https://buck.build
[Pants]: https://www.pantsbuild.org
[Please]: https://please.build

<br>

## Example

The following example serializes a minimal Bazel target for the `syn` crate.

The _tests/bazel.rs_ test in this repo has a somewhat more fleshed out example
of this use case, including things like `load(…)`, `package(default_visibility =
…)`, distinct `include` and `exclude` arguments to `glob(…)`, and `select({…})`.

```rust
#[derive(Serialize)]
#[serde(rename = "rust_library")]
pub struct RustLibrary {
    pub name: String,
    pub srcs: Glob,
    pub crate_features: BTreeSet<String>,
    pub edition: u16,
    pub deps: BTreeSet<String>,
}

#[derive(Serialize)]
#[serde(rename = "glob")]
pub struct Glob(pub BTreeSet<String>);

fn main() {
    let rust_library = RustLibrary { ... };

    print!("{}", serde_starlark::to_string(&rust_library).unwrap());
}
```

```bzl
rust_library(
    name = "syn",
    srcs = glob(["**/*.rs"]),
    crate_features = [
        "default",
        "full",
    ],
    edition = 2018,
    deps = [
        ":proc-macro2",
        ":quote",
        ":unicode-ident",
    ],
)
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
