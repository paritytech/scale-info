// Copyright 2019-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

use scale_info::prelude::{
    boxed::Box,
    marker::PhantomData,
    string::String,
    vec,
    vec::Vec,
};

use pretty_assertions::{
    assert_eq,
    assert_ne,
};
use scale::Encode;
use scale_info::{
    form::PortableForm,
    meta_type,
    IntoPortable as _,
    PortableRegistry,
    Registry,
    TypeInfo,
};
use serde_json::json;

fn assert_json_for_type<T>(expected_json: serde_json::Value)
where
    T: TypeInfo + ?Sized,
{
    let mut registry = Registry::new();

    let ty = T::type_info().into_portable(&mut registry);

    assert_eq!(serde_json::to_value(ty).unwrap(), expected_json,);
}

#[test]
fn test_primitives() {
    assert_json_for_type::<bool>(json!({ "def": { "primitive": "bool" } }));
    assert_json_for_type::<char>(json!({ "def": { "primitive": "char" } }));
    assert_json_for_type::<u8>(json!({ "def": { "primitive": "u8" } }));
    assert_json_for_type::<u16>(json!({ "def": { "primitive": "u16" } }));
    assert_json_for_type::<u32>(json!({ "def": { "primitive": "u32" } }));
    assert_json_for_type::<u64>(json!({ "def": { "primitive": "u64" } }));
    assert_json_for_type::<u128>(json!({ "def": { "primitive": "u128" } }));
    assert_json_for_type::<i16>(json!({ "def": { "primitive": "i16" } }));
    assert_json_for_type::<i32>(json!({ "def": { "primitive": "i32" } }));
    assert_json_for_type::<i64>(json!({ "def": { "primitive": "i64" } }));
    assert_json_for_type::<i128>(json!({ "def": { "primitive": "i128" } }));
}

#[test]
fn test_builtins() {
    // arrays
    assert_json_for_type::<[u8; 2]>(
        json!({ "def": { "array": { "len": 2, "type": 0 } } }),
    );
    assert_json_for_type::<[bool; 4]>(
        json!({ "def": { "array": { "len": 4, "type": 0 } } }),
    );
    assert_json_for_type::<[char; 8]>(
        json!({ "def": { "array": { "len": 8, "type": 0 } } }),
    );
    // tuples
    assert_json_for_type::<(u8, bool)>(json!({ "def": { "tuple": [ 0, 1 ] } }));
    assert_json_for_type::<(u8, bool, char, u128)>(
        json!({ "def": { "tuple": [ 0, 1, 2, 3 ] } }),
    );
    assert_json_for_type::<(u8, bool, char, u128, i32, u32)>(json!({
        "def": {
            "tuple": [ 0, 1, 2, 3, 4, 5 ]
        }
    }));
    // sequences
    assert_json_for_type::<[bool]>(json!({ "def": { "sequence": { "type": 0 } } }));
    assert_json_for_type::<&[bool]>(json!({ "def": { "sequence": { "type": 0 } } }));
    assert_json_for_type::<Vec<bool>>(json!({ "def": { "sequence": { "type": 0 } } }));
    // complex types
    assert_json_for_type::<Option<&str>>(json!({
        "path": ["Option"],
        "params": [0],
        "def": {
            "variant": {
                "variants": [
                    {
                        "name": "None",
                    },
                    {
                        "name": "Some",
                        "fields": [ { "type": 0 } ]
                    },
                ]
            }
        }
    }));
    assert_json_for_type::<Result<u32, u64>>(json!({
        "path": ["Result"],
        "params": [0, 1],
        "def": {
            "variant": {
                "variants": [
                    {
                        "name": "Ok",
                        "fields": [ { "type": 0 } ]
                    },
                    {
                        "name": "Err",
                        "fields": [ { "type": 1 } ]
                    }
                ]
            }
        }
    }));
    // references
    assert_json_for_type::<&bool>(json!({ "def": { "primitive": "bool" } }));
    assert_json_for_type::<&mut str>(json!({ "def": { "primitive": "str" } }));
    assert_json_for_type::<Box<u32>>(json!({ "def": { "primitive": "u32" } }));
    // strings
    assert_json_for_type::<String>(json!({ "def": { "primitive": "str" } }));
    assert_json_for_type::<str>(json!({ "def": { "primitive": "str" } }));
    // PhantomData
    assert_json_for_type::<PhantomData<bool>>(
        json!({ "def": { "phantom": { "type": 0 } }, }),
    )
}

#[test]
fn test_unit_struct() {
    #[derive(TypeInfo)]
    struct UnitStruct;

    assert_json_for_type::<UnitStruct>(json!({
        "path": ["json", "UnitStruct"],
        "def": {
            "composite": {},
        }
    }));
}

#[test]
fn test_tuplestruct() {
    #[derive(TypeInfo)]
    struct TupleStruct(i32, [u8; 32], bool);

    assert_json_for_type::<TupleStruct>(json!({
        "path": ["json", "TupleStruct"],
        "def": {
            "composite": {
                "fields": [
                    { "type": 0, "typeName": "i32" },
                    { "type": 1, "typeName": "[u8; 32]" },
                    { "type": 3, "typeName": "bool" },
                ],
            },
        }
    }));
}

#[test]
fn test_struct() {
    #[derive(TypeInfo)]
    struct Struct {
        a: i32,
        b: [u8; 32],
        c: bool,
    }

    assert_json_for_type::<Struct>(json!({
        "path": ["json", "Struct"],
        "def": {
            "composite": {
                "fields": [
                    { "name": "a", "type": 0, "typeName": "i32" },
                    { "name": "b", "type": 1, "typeName": "[u8; 32]" },
                    { "name": "c", "type": 3, "typeName": "bool" },
                ],
            },
        }
    }));
}

#[test]
fn test_struct_with_some_fields_marked_as_compact() {
    use scale::Encode;

    // #[derive(TypeInfo, Encode)]
    #[derive(Encode)]
    struct Dense {
        #[codec(compact)]
        a: u128,
        a_not_compact: u128,
        b: [u8; 32],
        #[codec(compact)]
        c: u64,
    }
    use scale_info::{
        build::Fields,
        Path,
        Type,
    };
    impl TypeInfo for Dense {
        type Identity = Self;
        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("Dense", module_path!()))
                .composite(
                    Fields::named()
                        .field(|f| f.compact::<u128>().name("a").type_name("u128"))
                        .field(|f| f.ty::<u128>().name("a_not_compact").type_name("u128"))
                        .field(|f| f.ty::<[u8; 32]>().name("b").type_name("[u8; 32]"))
                        .field(|f| f.compact::<u64>().name("c").type_name("u64")),
                )
        }
    }
    assert_json_for_type::<Dense>(json![{
        "path": ["json", "Dense"],
        "def": {
            "composite": {
                "fields": [
                    { "name": "a", "type": 0, "typeName": "u128" },
                    { "name": "a_not_compact", "type": 1, "typeName": "u128" },
                    { "name": "b", "type": 2, "typeName": "[u8; 32]" },
                    { "name": "c", "type": 4, "typeName": "u64" },
                ],
            },
        }
    }]);
}

#[test]
fn test_struct_with_phantom() {
    use scale_info::prelude::marker::PhantomData;
    #[derive(TypeInfo)]
    struct Struct<T> {
        a: i32,
        b: PhantomData<T>,
    }

    assert_json_for_type::<Struct<u8>>(json!({
        "path": ["json", "Struct"],
        "params": [0],
        "def": {
            "composite": {
                "fields": [
                    { "name": "a", "type": 1, "typeName": "i32" },
                    // type 1 is the `u8` in the `PhantomData`
                    { "name": "b", "type": 2, "typeName": "PhantomData<T>" },
                ],
            },
        }
    }));
}

#[test]
fn test_clike_enum() {
    #[derive(TypeInfo)]
    enum ClikeEnum {
        A,
        B = 42,
        C,
    }

    assert_json_for_type::<ClikeEnum>(json!({
        "path": ["json", "ClikeEnum"],
        "def": {
            "variant": {
                "variants": [
                    { "name": "A", "discriminant": 0, },
                    { "name": "B", "discriminant": 42, },
                    { "name": "C", "discriminant": 2, },
                ],
            },
        }
    }));
}

#[test]
fn test_enum() {
    #[derive(TypeInfo)]
    enum Enum {
        ClikeVariant,
        TupleStructVariant(u32, bool),
        StructVariant { a: u32, b: [u8; 32], c: char },
    }

    assert_json_for_type::<Enum>(json!({
        "path": ["json", "Enum"],
        "def": {
            "variant": {
                "variants": [
                    { "name": "ClikeVariant" },
                    {
                        "name": "TupleStructVariant",
                        "fields": [
                            { "type": 0, "typeName": "u32" },
                            { "type": 1, "typeName": "bool" },
                        ],
                    },
                    {
                        "name": "StructVariant",
                        "fields": [
                            { "name": "a", "type": 0, "typeName": "u32" },
                            { "name": "b", "type": 2, "typeName": "[u8; 32]" },
                            { "name": "c", "type": 4, "typeName": "char" },
                        ],
                    }
                ],
            },
        }
    }));
}

#[test]
fn enums_with_scale_indexed_variants() {
    #[derive(TypeInfo, Encode)]
    enum Animal {
        #[codec(index = 123)]
        Ape(u8),
        #[codec(index = 12)]
        Boar { a: u16, b: u32 },
        #[codec(index = 1)]
        Cat,
        #[codec(index = 0)]
        Dog(u64, u128),
    }

    assert_json_for_type::<Animal>(json!({
        "path": ["json", "Animal"],
        "def": {
            "variant": {
                "variants": [
                    {
                        "name": "Ape",
                        "index": 123,
                        "fields": [
                            { "type": 1, "typeName": "u8" }
                        ]
                    },
                    {
                        "name": "Boar",
                        "index": 12,
                        "fields": [
                            { "name": "a", "type": 2, "typeName": "u16" },
                            { "name": "b", "type": 3, "typeName": "u32" }
                        ]
                    },
                    {
                        "name": "Cat",
                        "index": 1,
                    },
                    {
                        "name": "Dog",
                        "index": 0,
                        "fields": [
                            { "type": 4, "typeName": "u64" },
                            { "type": 5, "typeName": "u128" }
                        ]
                    }
                ]
            }
        }
    }));
}

#[test]
fn test_recursive_type_with_box() {
    #[derive(TypeInfo)]
    pub enum Tree {
        Leaf { value: i32 },
        Node { right: Box<Tree>, left: Box<Tree> },
    }

    let mut registry = Registry::new();
    registry.register_type(&meta_type::<Tree>());

    let expected_json = json!({
        "types": [
            {
                "path": ["json", "Tree"],
                "def": {
                    "variant": {
                        "variants": [
                            {
                                "name": "Leaf",
                                "fields": [
                                    { "name": "value", "type": 1, "typeName": "i32" },
                                ],
                            },
                            {
                                "name": "Node",
                                "fields": [
                                    { "name": "right", "type": 0, "typeName": "Box<Tree>" },
                                    { "name": "left", "type": 0, "typeName": "Box<Tree>" },
                                ],
                            }
                        ],
                    },
                }
            },
            {
                "def": { "primitive": "i32" },
            },
        ]
    });

    let registry: PortableRegistry = registry.into();
    assert_eq!(serde_json::to_value(registry).unwrap(), expected_json,);
}

#[test]
fn registry_knows_about_compact_types() {
    #[allow(unused)]
    #[derive(TypeInfo, Encode)]
    struct Dense {
        #[codec(compact)]
        a: u128,
        a_not_compact: u128,
        b: [u8; 32],
        #[codec(compact)]
        c: u64,
    }
    let mut registry = Registry::new();
    let type_id = registry.register_type(&meta_type::<Dense>());

    let expected_json = json!({
        "types": [
            { // type 1
                "path": ["json", "Dense"],
                "def": {
                    "composite": {
                        "fields": [
                            { "name": "a", "type": 1, "typeName": "u128" },
                            { "name": "a_not_compact", "type": 2, "typeName": "u128" },
                            { "name": "b", "type": 3, "typeName": "[u8; 32]" },
                            { "name": "c", "type": 5, "typeName": "u64" }
                        ]
                    }
                }
            },
            { // type 2, the `Compact<u128>` of field `a`.
                "def": { "compact": { "type": 2 } },
            },
            { // type 3, the `u128` used by type 2 and field `a_not_compact`.
                "def": { "primitive": "u128" }
            },
            { // type 4, the `[u8; 32]` of field `b`.
                "def": { "array": { "len": 32, "type": 4 }}
            },
            { // type 5, the `u8` in `[u8; 32]`
                "def": { "primitive": "u8" }
            },
            { // type 6, the `Compact<u64>` of field `c`
                "def": { "compact": { "type": 6 } },
            },
            { // type 7, the `u64` in `Compact<u64>` of field `c`
                "def": { "primitive": "u64" }
            },
        ]
    });

    let registry: PortableRegistry = registry.into();
    assert_eq!(serde_json::to_value(registry).unwrap(), expected_json,);
}

#[test]
fn test_registry() {
    let mut registry = Registry::new();

    #[derive(TypeInfo)]
    struct UnitStruct;
    #[derive(TypeInfo)]
    struct TupleStruct(u8, u32);
    #[derive(TypeInfo)]
    struct Struct {
        a: u8,
        b: u32,
        c: [u8; 32],
    }
    #[derive(TypeInfo)]
    struct RecursiveStruct {
        rec: Vec<RecursiveStruct>,
    }
    #[derive(TypeInfo)]
    enum ClikeEnum {
        A,
        B,
        C,
    }
    #[derive(TypeInfo)]
    enum RustEnum {
        A,
        B(u8, u32),
        C { a: u8, b: u32, c: [u8; 32] },
    }

    registry.register_type(&meta_type::<UnitStruct>());
    registry.register_type(&meta_type::<TupleStruct>());
    registry.register_type(&meta_type::<Struct>());
    registry.register_type(&meta_type::<RecursiveStruct>());
    registry.register_type(&meta_type::<ClikeEnum>());
    registry.register_type(&meta_type::<RustEnum>());

    let expected_json = json!({
        "types": [
            { // type 0
                "path": [
                    "json",
                    "UnitStruct",
                ],
                "def": {
                    "composite": {},
                }
            },
            { // type 1
                "path": [
                    "json",
                    "TupleStruct",
                ],
                "def": {
                    "composite": {
                        "fields": [
                            { "type": 2, "typeName": "u8" },
                            { "type": 3, "typeName": "u32" },
                        ],
                    },
                }
            },
            { // type 2
                "def": { "primitive": "u8" },
            },
            { // type 3
                "def": { "primitive": "u32" },
            },
            { // type 4
                "path": [
                    "json",
                    "Struct",
                ],
                "def": {
                    "composite": {
                        "fields": [
                            {
                                "name": "a",
                                "type": 2,
                                "typeName": "u8"
                            },
                            {
                                "name": "b",
                                "type": 3,
                                "typeName": "u32"
                            },
                            {
                                "name": "c",
                                "type": 5,
                                "typeName": "[u8; 32]"
                            }
                        ]
                    },
                }
            },
            { // type 5
                "def": {
                    "array": {
                        "len": 32,
                        "type": 2, // u8
                    },
                }
            },
            { // type 6
                "path": [
                    "json",
                    "RecursiveStruct",
                ],
                "def": {
                    "composite": {
                        "fields": [
                            {
                                "name": "rec",
                                "type": 7,
                                "typeName": "Vec<RecursiveStruct>"
                            }
                        ]
                    },
                }
            },
            { // type 7
                "def": {
                    "sequence": {
                        "type": 6, // RecursiveStruct
                    },
                }
            },
            { // type 8
                "path": [
                    "json",
                    "ClikeEnum",
                ],
                "def": {
                    "variant": {
                        "variants": [
                            {
                                "name": "A",
                                "discriminant": 0,
                            },
                            {
                                "name": "B",
                                "discriminant": 1,
                            },
                            {
                                "name": "C",
                                "discriminant": 2,
                            },
                        ]
                    }
                }
            },
            { // type 9
                "path": [
                    "json",
                    "RustEnum"
                ],
                "def": {
                    "variant": {
                        "variants": [
                            {
                                "name": "A"
                            },
                            {
                                "name": "B",
                                "fields": [
                                    { "type": 2, "typeName": "u8" }, // u8
                                    { "type": 3, "typeName": "u32" }, // u32
                                ]
                            },
                            {
                                "name": "C",
                                "fields": [
                                    {
                                        "name": "a",
                                        "type": 2, // u8
                                        "typeName": "u8"
                                    },
                                    {
                                        "name": "b",
                                        "type": 3, // u32
                                        "typeName": "u32"
                                    },
                                    {
                                        "name": "c",
                                        "type": 5,
                                        "typeName": "[u8; 32]"
                                    }
                                ]
                            }
                        ]
                    },
                }
            },
        ]
    });

    let registry: PortableRegistry = registry.into();
    assert_eq!(serde_json::to_value(registry).unwrap(), expected_json,);
}
