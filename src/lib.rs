//! [![github]](https://github.com/dtolnay/serde-starlark)&ensp;[![crates-io]](https://crates.io/crates/serde_starlark)&ensp;[![docs-rs]](https://docs.rs/serde_starlark)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! Serde serializer for generating syntactically valid Starlark, the
//! declarative format used for describing build targets in build systems
//! including [Bazel], [Buck], [Pants], and [Please].
//!
//! [Bazel]: https://bazel.build
//! [Buck]: https://buck.build
//! [Pants]: https://www.pantsbuild.org
//! [Please]: https://please.build
//!
//! # Example
//!
//! The following example serializes a minimal Bazel target for the `syn` crate.
//!
//! The _tests/bazel.rs_ test in this repo has a somewhat more fleshed out
//! example of this use case, including things like `load(…)`,
//! `package(default_visibility = …)`, distinct `include` and `exclude`
//! arguments to `glob(…)`, and `select({…})`.
//!
//! ```
//! # use serde_derive::Serialize;
//! # use std::collections::BTreeSet;
//! #
//! #[derive(Serialize)]
//! #[serde(rename = "rust_library")]
//! pub struct RustLibrary {
//!     pub name: String,
//!     pub srcs: Glob,
//!     pub crate_features: BTreeSet<String>,
//!     pub edition: u16,
//!     pub deps: BTreeSet<String>,
//! }
//!
//! #[derive(Serialize)]
//! #[serde(rename = "glob")]
//! pub struct Glob(pub BTreeSet<String>);
//!
//! fn main() {
//! #   let rust_library = RustLibrary {
//! #       name: "syn".to_owned(),
//! #       srcs: Glob(BTreeSet::from(["**/*.rs".to_owned()])),
//! #       crate_features: BTreeSet::from(["default".to_owned(), "full".to_owned()]),
//! #       edition: 2018,
//! #       deps: BTreeSet::from([
//! #           ":proc-macro2".to_owned(),
//! #           ":quote".to_owned(),
//! #           ":unicode-ident".to_owned(),
//! #       ]),
//! #   };
//! #   let _ = stringify! {
//!     let rust_library = RustLibrary { ... };
//! #   };
//!
//!     print!("{}", serde_starlark::to_string(&rust_library).unwrap());
//! }
//! ```
//!
//! ```bzl
//! rust_library(
//!     name = "syn",
//!     srcs = glob(["**/*.rs"]),
//!     crate_features = [
//!         "default",
//!         "full",
//!     ],
//!     edition = 2018,
//!     deps = [
//!         ":proc-macro2",
//!         ":quote",
//!         ":unicode-ident",
//!     ],
//! )
//! ```
//!
//! # Data model
//!
//! The primitive types (integers, boolean, string) serialize in the obvious way
//! to Starlark.
//!
//! Serde sequences serialize to Starlark arrays. Serde maps serialize to
//! Starlark maps.
//!
//! Rust structs with named fields serialize to Starlark "function calls" with
//! named arguments:
//!
//! ```
//! # use serde_derive::Serialize;
//! #
//! #[derive(Serialize)]
//! #[serde(rename = "rust_library")]
//! pub struct RustLibrary {
//!     pub name: String,
//!     pub edition: u16,
//! }
//! ```
//!
//! ```bzl
//! rust_library(
//!     name = "syn",
//!     edition = 2018,
//! )
//! ```
//!
//! Rust newtype structs and tuple structs serialize to Starlark "function
//! calls" with positional arguments:
//!
//! ```
//! # use serde_derive::Serialize;
//! # use std::collections::BTreeMap;
//! #
//! #[derive(Serialize)]
//! #[serde(rename = "select")]
//! pub struct Select<T>(pub BTreeMap<String, T>);
//! ```
//!
//! ```bzl
//! select({
//!     "//conditions:default": [],
//! })
//! ```
//!
//! To make a newtype struct which does not appear as a function call, use the
//! `serde(transparent)` attribute.
//!
//! ```
//! # use serde_derive::Serialize;
//! #
//! #[derive(Serialize)]
//! #[serde(transparent)]
//! pub struct Dependency(pub String);
//! ```
//!
//! Fields of type `Option<T>` serialize as either `None` or the value if
//! present. Consider using `serde(skip_serializing_if = "Option::is_none")` to
//! omit fields with value `None` from the serialized output.

#![doc(html_root_url = "https://docs.rs/serde_starlark/0.1.16")]
#![allow(
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::needless_doctest_main,
    clippy::struct_excessive_bools,
    clippy::uninlined_format_args
)]

mod assignment;
mod call;
mod comment;
mod error;
mod ser;

use crate::ser::{WriteMap, WriteSeq, WriteStarlark, WriteStruct, WriteTuple, WriteTupleStruct};
use serde::ser::{Impossible, Serialize};

/// For "deserialization", consider using <https://github.com/facebookexperimental/starlark-rust>.
#[cfg(doc)]
pub mod de {}

pub struct Error {
    kind: crate::error::ErrorKind,
}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Serialize,
{
    value.serialize(Serializer)
}

/// Format a function call, array, or map with all values on one line.
///
/// # Defaults
///
/// The default newline behavior of derived Serialize impls is:
///
/// - function calls with named arguments: always multi-line
/// - function calls with positional arguments: one-line if 1 argument,
///   multi-line for more
/// - arrays: one-line if length 1, multi-line for length &gt;1
/// - tuples: always one-line
/// - maps: always multi-line
///
/// These formatting defaults can all be overridden using serde_starlark's
/// ONELINE and MULTILINE constants.
///
/// # Example
///
/// The derived impl produces a function call with named arguments in multi-line
/// format.
///
/// ```
/// use serde_derive::Serialize;
///
/// #[derive(Serialize)]
/// #[serde(rename = "crate")]
/// struct Crate {
///     name: String,
///     version: semver::Version,
/// }
///
/// fn main() {
///     let krate = Crate {
///         name: "serde_starlark".to_owned(),
///         version: semver::Version::new(1, 0, 0),
///     };
///
///     print!("{}", serde_starlark::to_string(&krate).unwrap());
/// }
/// ```
///
/// ```bzl
/// crate(
///     name = "serde_starlark",
///     version = "1.0.0",
/// )
/// ```
///
/// We can use ONELINE to serialize the call on one line.
///
/// ```
/// use serde::ser::{Serialize, SerializeStruct, Serializer};
///
/// struct Crate {
///     name: String,
///     version: semver::Version,
/// }
///
/// impl Serialize for Crate {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where
///         S: Serializer,
///     {
///         let mut call = serializer.serialize_struct("crate", serde_starlark::ONELINE)?;
///         call.serialize_field("name", &self.name)?;
///         call.serialize_field("version", &self.version)?;
///         call.end()
///     }
/// }
///
/// fn main() {
///     let krate = Crate {
///         name: "serde_starlark".to_owned(),
///         version: semver::Version::new(1, 0, 0),
///     };
///
///     print!("{}", serde_starlark::to_string(&krate).unwrap());
/// }
/// ```
///
/// ```bzl
/// crate(name = "serde_starlark", version = "1.0.0")
/// ```
pub const ONELINE: usize = usize::MIN;

/// Format a function call, array, or map with all values on their own line.
///
/// # Defaults
///
/// The default newline behavior of derived Serialize impls is:
///
/// - function calls with named arguments: always multi-line
/// - function calls with positional arguments: one-line if 1 argument,
///   multi-line for more
/// - arrays: one-line if length 1, multi-line for length &gt;1
/// - tuples: always one-line
/// - maps: always multi-line
///
/// These formatting defaults can all be overridden using serde_starlark's
/// ONELINE and MULTILINE constants.
///
/// # Example
///
/// The derived impl produces an array of length 1 on one line.
///
/// ```
/// use serde_derive::Serialize;
///
/// #[derive(Serialize)]
/// #[serde(rename = "glob")]
/// struct Glob(Vec<String>);
///
/// fn main() {
///     let glob = Glob(vec!["**/*.rs".to_owned()]);
///     print!("{}", serde_starlark::to_string(&glob).unwrap());
/// }
/// ```
///
/// ```bzl
/// glob(["**/*.rs"])
/// ```
///
/// We can choose to use the MULTILINE format for the array instead.
///
/// ```
/// use serde::ser::{Serialize, SerializeSeq, Serializer};
/// use serde_derive::Serialize;
///
/// #[derive(Serialize)]
/// #[serde(rename = "glob")]
/// struct Glob(#[serde(serialize_with = "multiline")] Vec<String>);
///
/// fn multiline<T, S>(array: &[T], serializer: S) -> Result<S::Ok, S::Error>
/// where
///     T: Serialize,
///     S: Serializer,
/// {
///     let mut seq = serializer.serialize_seq(Some(serde_starlark::MULTILINE))?;
///     for element in array {
///         seq.serialize_element(element)?;
///     }
///     seq.end()
/// }
///
/// fn main() {
///     let glob = Glob(vec!["**/*.rs".to_owned()]);
///     print!("{}", serde_starlark::to_string(&glob).unwrap());
/// }
/// ```
///
/// ```bzl
/// glob([
///     "**/*.rs",
/// ])
/// ```
pub const MULTILINE: usize = usize::MAX;

/// Serialize a value as an assigment to an identifier.
///
/// # Example
///
/// Assigning a simple value to an identifier.
///
/// ```
/// # use std::collections::BTreeMap;
/// #
/// use serde_starlark::Assignment;
///
/// let version = Assignment::new("VERSION", "1.0.0");
/// print!("{}", serde_starlark::to_string(&version).unwrap());
///
/// let metadata = Assignment::new(
///     "METADATA",
///     BTreeMap::from([("name", "project"), ("version", "1.0.0")]),
/// );
/// print!("{}", serde_starlark::to_string(&metadata).unwrap());
/// #
/// # assert_eq!(
/// #   serde_starlark::to_string(&version).unwrap(),
/// #   "VERSION = \"1.0.0\"\n",
/// # );
/// # assert_eq!(
/// #   serde_starlark::to_string(&metadata).unwrap(),
/// #   concat!(
/// #       "METADATA = {\n",
/// #       "    \"name\": \"project\",\n",
/// #       "    \"version\": \"1.0.0\",\n",
/// #       "}\n",
/// #   ),
/// # );
/// ```
///
/// Produces:
///
/// ```bzl
/// VERSION = "1.0.0"
/// METADATA = {
///     "name": "project",
///     "version": "1.0.0",
/// }
/// ```
pub struct Assignment<'identifier, T> {
    identifier: &'identifier str,
    value: T,
}

impl<'identifier, T> Assignment<'identifier, T> {
    pub fn new(identifier: &'identifier str, value: T) -> Self {
        Assignment { identifier, value }
    }
}

/// Serialize a map as a function call.
///
/// Primarily this becomes involved when using the `serde(flatten)` attribute.
/// For example:
///
/// ```
/// # use serde_derive::Serialize;
/// #
/// pub enum Rule {
///     RustLibrary(RustLibrary),
///     RustBinary(RustBinary),
/// }
///
/// #[derive(Serialize)]
/// pub struct RustLibrary {
///     #[serde(flatten)]
///     common: RustCommon,
///
///     proc_macro: bool,
/// }
///
/// #[derive(Serialize)]
/// pub struct RustBinary {
///     #[serde(flatten)]
///     common: RustCommon,
/// }
///
/// #[derive(Serialize)]
/// pub struct RustCommon {
///     name: String,
///     deps: Vec<String>,
///     // ...
/// }
/// ```
///
/// Normally, structs with named fields get serialized as function calls with
/// named arguments by serde_starlark. However a quirk of `serde(flatten)` is
/// that Serde processes structs containing this attribute as if they were maps,
/// not structs. In Starlark unlike in JSON, maps and structs are differently
/// serialized, so we would need to use a `FunctionCall` in this situation to
/// ensure we get a function call in the serialized output, not a map.
///
/// ```
/// # use serde_derive::Serialize;
/// #
/// # pub enum Rule {
/// #     RustLibrary(RustLibrary),
/// #     RustBinary(RustBinary),
/// # }
/// #
/// # #[derive(Serialize)]
/// # pub struct RustLibrary {}
/// # #[derive(Serialize)]
/// # pub struct RustBinary {}
/// #
/// use serde::ser::{Serialize, Serializer};
/// use serde_starlark::FunctionCall;
///
/// impl Serialize for Rule {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where
///         S: Serializer,
///     {
///         match self {
///             Rule::RustLibrary(library) => {
///                 FunctionCall::new("rust_library", library).serialize(serializer)
///             }
///             Rule::RustBinary(binary) => {
///                 FunctionCall::new("rust_binary", binary).serialize(serializer)
///             }
///         }
///     }
/// }
/// ```
pub struct FunctionCall<'name, A> {
    function: &'name str,
    args: A,
}

impl<'name, A> FunctionCall<'name, A> {
    pub fn new(function: &'name str, args: A) -> Self {
        FunctionCall { function, args }
    }
}

/// Serialize a line comment on the end of the current line.
///
/// # Example
///
/// This example demonstrates serializing a `select({…})` in which the keys are
/// Bazel platforms, but we want to preserve the original Cargo conditional
/// dependency cfg expression as a comment.
///
/// For example we had a Cargo.toml containing this:
///
/// ```toml
/// [target.'cfg(any(unix, target_os = "wasi"))'.dependencies]
/// libc = "0.2"
///
/// [target.'cfg(windows)'.dependencies]
/// windows-sys = "0.42"
/// ```
///
/// And we want to get the following Starlark `select`:
///
/// ```bzl
/// deps = select({
///     "@rules_rust//rust/platform:x86_64-pc-windows-msvc": [
///         "//third-party/rust:windows-sys",  # cfg(windows)
///     ],
///     "@rules_rust//rust/platform:x86_64-unknown-linux-gnu": [
///         "//third-party/rust:libc",  # cfg(any(unix, target_os = "wasi"))
///     ],
///     "@rules_rust//rust/platform:wasm32-wasi": [
///         "//third-party/rust:libc",  # cfg(any(unix, target_os = "wasi"))
///     ],
/// })
/// ```
///
/// ```
/// use serde::ser::{Serialize, SerializeSeq, Serializer};
/// use serde_starlark::{FunctionCall, LineComment};
/// use std::collections::{BTreeMap, BTreeSet};
///
/// #[derive(Ord, PartialOrd, Eq, PartialEq)]
/// pub struct WithCargoCfg<T> {
///     value: T,
///     cfg: String,
/// }
///
/// impl<T> Serialize for WithCargoCfg<T>
/// where
///     T: Serialize,
/// {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where
///         S: Serializer,
///     {
///         LineComment::new(&self.value, &self.cfg).serialize(serializer)
///     }
/// }
///
/// // Serialize an array with each element on its own line, even if there
/// // is just a single element which serde_starlark would ordinarily place
/// // on the same line as the array brackets.
/// struct MultilineArray<A>(A);
///
/// impl<A, T> Serialize for MultilineArray<A>
/// where
///     for<'a> &'a A: IntoIterator<Item = &'a T>,
///     T: Serialize,
/// {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where
///         S: Serializer,
///     {
///         let mut array = serializer.serialize_seq(Some(serde_starlark::MULTILINE))?;
///         for element in &self.0 {
///             array.serialize_element(element)?;
///         }
///         array.end()
///     }
/// }
///
/// fn main() {
///     let deps = BTreeMap::from([
///         (
///             "@rules_rust//rust/platform:x86_64-pc-windows-msvc".to_owned(),
///             MultilineArray(BTreeSet::from([WithCargoCfg {
///                 value: "//third-party/rust:windows-sys",
///                 cfg: "cfg(windows)".to_owned(),
///             }])),
///         ),
///         (
///             "@rules_rust//rust/platform:x86_64-unknown-linux-gnu".to_owned(),
///             MultilineArray(BTreeSet::from([WithCargoCfg {
///                 value: "//third-party/rust:libc",
///                 cfg: "cfg(any(unix, target_os = \"wasi\"))".to_owned(),
///             }])),
///         ),
///     ]);
///
///     let select = FunctionCall::new("select", (deps,));
///     print!("{}", serde_starlark::to_string(&select).unwrap());
/// }
/// ```
pub struct LineComment<'comment, T> {
    value: T,
    comment: &'comment str,
}

impl<'comment, T> LineComment<'comment, T> {
    pub fn new(value: T, comment: &'comment str) -> Self {
        assert!(!comment.starts_with('#'));
        assert!(!comment.contains('\n'));
        LineComment { value, comment }
    }
}

/// Serializer whose output `Ok` type is Starlark.
///
/// `value.serialize(serde_starlark::Serializer)` is 100% equivalent to
/// `serde_starlark::to_string(&value)`. However, having direct access to the
/// Serializer is useful when dealing with libraries that act as Serializer
/// adapters, such as the erased-serde crate or serde-transcode crate.
pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = WriteSeq<WriteStarlark>;
    type SerializeTuple = WriteTuple<WriteStarlark>;
    type SerializeTupleStruct = WriteTupleStruct<WriteStarlark>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = WriteMap<WriteStarlark>;
    type SerializeStruct = WriteStruct<WriteStarlark>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i64(v)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i128(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u64(v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u128(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_none()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new().serialize_some(value)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new().serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new().serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        WriteStarlark::new().serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        WriteStarlark::new().serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        WriteStarlark::new().serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        WriteStarlark::new().serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        WriteStarlark::new().serialize_map(len)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        WriteStarlark::new().serialize_struct(name, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        WriteStarlark::new().serialize_struct_variant(name, variant_index, variant, len)
    }
}
