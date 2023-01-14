use expect_test::expect;
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
    pub items: Set<String>,
}

pub struct Package {
    pub default_visibility: Set<String>,
}

#[derive(Serialize)]
#[serde(rename = "rust_library")]
pub struct RustLibrary {
    pub name: String,
    pub srcs: Glob,
    pub crate_features: Set<String>,
    pub edition: u16,
    pub proc_macro: bool,
    pub rustc_env: Map<String, String>,
    #[serde(serialize_with = "serialize_select")]
    pub deps: Map<Condition, Set<String>>,
}

pub struct Glob {
    pub include: Set<String>,
    pub exclude: Set<String>,
}

#[derive(Serialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(transparent)]
pub struct Condition(pub &'static str);

impl Serialize for Load {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple_struct("load", 0)?;
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
            serializer.serialize_newtype_struct("glob", &self.include)
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
    serializer.serialize_newtype_struct("select", deps)
}

#[test]
fn test_struct() {
    let build_syn = vec![
        Rule::Load(Load {
            bzl: "@rules_rust//rust:defs.bzl".to_owned(),
            items: Set::from(["rust_library".to_owned()]),
        }),
        Rule::Package(Package {
            default_visibility: Set::from(["//visibility:public".to_owned()]),
        }),
        Rule::RustLibrary(RustLibrary {
            name: "syn".to_owned(),
            srcs: Glob {
                include: Set::from(["**/*.rs".to_owned()]),
                exclude: Set::new(),
            },
            crate_features: Set::from(["default".to_owned(), "full".to_owned()]),
            edition: 2018,
            proc_macro: false,
            rustc_env: Map::new(),
            deps: Map::from([(
                Condition("//conditions:default"),
                Set::from([
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

    let expected = expect![[r#"
        load("@rules_rust//rust:defs.bzl", "rust_library")

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
    "#]];

    expected.assert_eq(&starlark);
}
