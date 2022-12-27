use insta::assert_display_snapshot;
use itertools::Itertools;
use serde::ser::{Serialize, SerializeStruct, SerializeTupleStruct, Serializer};
use serde_derive::Serialize;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Rule {
    Load(Load),
    Package(Package),
    RustLibrary(RustLibrary),
}

pub struct Load {
    pub bzl: String,
    pub items: Vec<String>,
}

pub struct Package {
    pub default_visibility: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename = "rust_library")]
pub struct RustLibrary {
    pub name: String,
    pub srcs: Glob,
    pub crate_features: Vec<String>,
    pub edition: u16,
    pub proc_macro: bool,
    pub rustc_env: Map<String, String>,
    #[serde(serialize_with = "serialize_select")]
    pub deps: Map<Condition, Set<String>>,
}

pub struct Glob {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(transparent)]
pub struct Condition(pub &'static str);

impl Serialize for Load {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple_struct("load", 1 + self.items.len())?;
        s.serialize_field(&self.bzl)?;
        for item in &self.items {
            s.serialize_field(item)?;
        }
        s.end()
    }
}

impl Serialize for Package {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("package", 0)?;
        s.serialize_field("default_visibility", &self.default_visibility)?;
        s.end()
    }
}

impl Serialize for Glob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.exclude.is_empty() {
            let mut s = serializer.serialize_tuple_struct("glob", 1)?;
            s.serialize_field(&self.include)?;
            s.end()
        } else {
            let mut s = serializer.serialize_struct("glob", 2)?;
            s.serialize_field("include", &self.include)?;
            s.serialize_field("exclude", &self.exclude)?;
            s.end()
        }
    }
}

fn serialize_select<K, V, S>(deps: &Map<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    V: Serialize,
    S: Serializer,
{
    let mut s = serializer.serialize_tuple_struct("select", 1)?;
    s.serialize_field(deps)?;
    s.end()
}

#[test]
fn test_struct() {
    let build_syn = vec![
        Rule::Load(Load {
            bzl: "@rules_rust//rust:defs.bzl".to_owned(),
            items: vec!["rust_library".to_owned()],
        }),
        Rule::Package(Package {
            default_visibility: vec!["//visibility:public".to_owned()],
        }),
        Rule::RustLibrary(RustLibrary {
            name: "syn".to_owned(),
            srcs: Glob {
                include: vec!["**/*.rs".to_owned()],
                exclude: Vec::new(),
            },
            crate_features: vec!["default".to_owned(), "full".to_owned()],
            edition: 2018,
            proc_macro: false,
            rustc_env: Map::new(),
            deps: Map::from_iter([(
                Condition("//conditions:default"),
                Set::from_iter([
                    ":proc-macro2".to_owned(),
                    ":quote".to_owned(),
                    ":unicode-ident".to_owned(),
                ]),
            )]),
        }),
    ];

    let starlark = build_syn
        .iter()
        .map(serde_starlark::to_string)
        .map(Result::unwrap)
        .join("\n");

    assert_display_snapshot!(starlark, @r###"
    load(
        "@rules_rust//rust:defs.bzl",
        "rust_library",
    )

    package(default_visibility = ["//visibility:public"])

    rust_library(
        name = "syn",
        srcs = glob(["**/*.rs"]),
        crate_features = [
            "default",
            "full",
        ],
        edition = 2018,
        proc_macro = False,
        rustc_env = {},
        deps = select({
            "//conditions:default": [
                ":proc-macro2",
                ":quote",
                ":unicode-ident",
            ],
        }),
    )
    "###);
}
