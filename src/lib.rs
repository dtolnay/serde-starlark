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
//! #       srcs: Glob(BTreeSet::from_iter(["**/*.rs".to_owned()])),
//! #       crate_features: BTreeSet::from_iter(["default".to_owned(), "full".to_owned()]),
//! #       edition: 2018,
//! #       deps: BTreeSet::from_iter([
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

#![doc(html_root_url = "https://docs.rs/serde_starlark/0.0.0")]
#![allow(
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::uninlined_format_args
)]

mod call;
mod error;
mod ser;

use crate::newline::WithNewline;
use crate::ser::{WriteMap, WriteSeq, WriteStarlark, WriteStruct, WriteTupleStruct};
use serde::ser::{Impossible, Serialize};

pub struct Error {
    kind: crate::error::ErrorKind,
}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Serialize,
{
    value.serialize(Serializer)
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
pub struct FunctionCall<A> {
    function: &'static str,
    args: A,
}

impl<A> FunctionCall<A> {
    pub fn new(function: &'static str, args: A) -> Self {
        FunctionCall { function, args }
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
    type SerializeSeq = WithNewline<WriteSeq<WriteStarlark>>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = WithNewline<WriteTupleStruct<WriteStarlark>>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = WithNewline<WriteMap<WriteStarlark>>;
    type SerializeStruct = WithNewline<WriteStruct<WriteStarlark>>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_bool(v).map(newline)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i8(v).map(newline)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i16(v).map(newline)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i32(v).map(newline)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i64(v).map(newline)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i128(v).map(newline)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u8(v).map(newline)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u16(v).map(newline)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u32(v).map(newline)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u64(v).map(newline)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u128(v).map(newline)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_f32(v).map(newline)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_f64(v).map(newline)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_char(v).map(newline)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_str(v).map(newline)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_bytes(v).map(newline)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_none().map(newline)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new().serialize_some(value).map(newline)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_unit().map(newline)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new()
            .serialize_unit_struct(name)
            .map(newline)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new()
            .serialize_unit_variant(name, variant_index, variant)
            .map(newline)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new()
            .serialize_newtype_struct(name, value)
            .map(newline)
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
        WriteStarlark::new()
            .serialize_newtype_variant(name, variant_index, variant, value)
            .map(newline)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        WriteStarlark::new().serialize_seq(len).map(WithNewline)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        WriteStarlark::new().serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        WriteStarlark::new()
            .serialize_tuple_struct(name, len)
            .map(WithNewline)
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
        WriteStarlark::new().serialize_map(len).map(WithNewline)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        WriteStarlark::new()
            .serialize_struct(name, len)
            .map(WithNewline)
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

fn newline(mut starlark: String) -> String {
    starlark.push('\n');
    starlark
}

mod newline {
    use super::newline;
    use serde::ser::{
        Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeTupleStruct,
    };

    pub struct WithNewline<S>(pub(crate) S);

    impl<S> SerializeSeq for WithNewline<S>
    where
        S: SerializeSeq<Ok = String>,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_element(value)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            self.0.end().map(newline)
        }
    }

    impl<S> SerializeTupleStruct for WithNewline<S>
    where
        S: SerializeTupleStruct<Ok = String>,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_field(value)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            self.0.end().map(newline)
        }
    }

    impl<S> SerializeMap for WithNewline<S>
    where
        S: SerializeMap<Ok = String>,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_key(key)
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_value(value)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            self.0.end().map(newline)
        }
    }

    impl<S> SerializeStruct for WithNewline<S>
    where
        S: SerializeStruct<Ok = String>,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_field(key, value)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            self.0.end().map(newline)
        }
    }
}
