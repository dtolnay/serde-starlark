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

<br>

## Data model

The primitive types (integers, boolean, string) serialize in the obvious way to
Starlark.

Serde sequences serialize to Starlark arrays. Serde maps serialize to Starlark
maps.

Rust structs with named fields serialize to Starlark "function calls" with named
arguments:

```rust
#[derive(Serialize)]
#[serde(rename = "rust_library")]
pub struct RustLibrary {
    pub name: String,
    pub edition: u16,
}
```

```bzl
rust_library(
    name = "syn",
    edition = 2018,
)
```

Rust newtype structs and tuple structs serialize to Starlark "function calls"
with positional arguments:

```rust
#[derive(Serialize)]
#[serde(rename = "select")]
pub struct Select<T>(pub BTreeMap<String, T>);
```

```bzl
select({
    "//conditions:default": [],
})
```

To make a newtype struct which does not appear as a function call, use the
`serde(transparent)` attribute.

```rust
#[derive(Serialize)]
#[serde(transparent)]
pub struct Dependency(pub String);
```

Fields of type `Option<T>` serialize as either `None` or the value if present.
Consider using `serde(skip_serializing_if = "Option::is_none")` to omit fields
with value `None` from the serialized output.

<br>

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
