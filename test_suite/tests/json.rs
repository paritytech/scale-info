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
        json!({ "def": { "array": { "len": 2, "type": 1 } } }),
    );
    assert_json_for_type::<[bool; 4]>(
        json!({ "def": { "array": { "len": 4, "type": 1 } } }),
    );
    assert_json_for_type::<[char; 8]>(
        json!({ "def": { "array": { "len": 8, "type": 1 } } }),
    );
    // tuples
    assert_json_for_type::<(u8, bool)>(json!({ "def": { "tuple": [ 1, 2 ] } }));
    assert_json_for_type::<(u8, bool, char, u128)>(
        json!({ "def": { "tuple": [ 1, 2, 3, 4 ] } }),
    );
    assert_json_for_type::<(u8, bool, char, u128, i32, u32)>(json!({
        "def": {
            "tuple": [ 1, 2, 3, 4, 5, 6 ]
        }
    }));
    // sequences
    assert_json_for_type::<[bool]>(json!({ "def": { "sequence": { "type": 1 } } }));
    assert_json_for_type::<&[bool]>(json!({ "def": { "sequence": { "type": 1 } } }));
    assert_json_for_type::<Vec<bool>>(json!({ "def": { "sequence": { "type": 1 } } }));
    // complex types
    assert_json_for_type::<Option<&str>>(json!({
        "path": ["Option"],
        "params": [1],
        "def": {
            "variant": {
                "variants": [
                    {
                        "name": "None",
                    },
                    {
                        "name": "Some",
                        "fields": [ { "type": 1, "typeName": "T" } ]
                    },
                ]
            }
        }
    }));
    assert_json_for_type::<Result<u32, u64>>(json!({
        "path": ["Result"],
        "params": [1, 2],
        "def": {
            "variant": {
                "variants": [
                    {
                        "name": "Ok",
                        "fields": [ { "type": 1, "typeName": "T" } ]
                    },
                    {
                        "name": "Err",
                        "fields": [ { "type": 2, "typeName": "E" } ]
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
    assert_json_for_type::<PhantomData<bool>>(json!({
        "path": ["PhantomData"],
        "def": { "phantom": { "type": 1 } },
        "params": [1]
    }))
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
                    { "type": 1, "typeName": "i32" },
                    { "type": 2, "typeName": "[u8; 32]" },
                    { "type": 4, "typeName": "bool" },
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
                    { "name": "a", "type": 1, "typeName": "i32" },
                    { "name": "b", "type": 2, "typeName": "[u8; 32]" },
                    { "name": "c", "type": 4, "typeName": "bool" },
                ],
            },
        }
    }));
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
        "params": [1],
        "def": {
            "composite": {
                "fields": [
                    { "name": "a", "type": 2, "typeName": "i32" },
                    // type 1 is the `u8` in the `PhantomData`
                    { "name": "b", "type": 3, "typeName": "PhantomData<T>" },
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
                            { "type": 1, "typeName": "u32" },
                            { "type": 2, "typeName": "bool" },
                        ],
                    },
                    {
                        "name": "StructVariant",
                        "fields": [
                            { "name": "a", "type": 1, "typeName": "u32" },
                            { "name": "b", "type": 3, "typeName": "[u8; 32]" },
                            { "name": "c", "type": 5, "typeName": "char" },
                        ],
                    }
                ],
            },
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
                                    { "name": "value", "type": 2, "typeName": "i32" },
                                ],
                            },
                            {
                                "name": "Node",
                                "fields": [
                                    { "name": "right", "type": 1, "typeName": "Box<Tree>" },
                                    { "name": "left", "type": 1, "typeName": "Box<Tree>" },
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
            { // type 1
                "path": [
                    "json",
                    "UnitStruct",
                ],
                "def": {
                    "composite": {},
                }
            },
            { // type 2
                "path": [
                    "json",
                    "TupleStruct",
                ],
                "def": {
                    "composite": {
                        "fields": [
                            { "type": 3, "typeName": "u8" },
                            { "type": 4, "typeName": "u32" },
                        ],
                    },
                }
            },
            { // type 3
                "def": { "primitive": "u8" },
            },
            { // type 4
                "def": { "primitive": "u32" },
            },
            { // type 5
                "path": [
                    "json",
                    "Struct",
                ],
                "def": {
                    "composite": {
                        "fields": [
                            {
                                "name": "a",
                                "type": 3,
                                "typeName": "u8"
                            },
                            {
                                "name": "b",
                                "type": 4,
                                "typeName": "u32"
                            },
                            {
                                "name": "c",
                                "type": 6,
                                "typeName": "[u8; 32]"
                            }
                        ]
                    },
                }
            },
            { // type 6
                "def": {
                    "array": {
                        "len": 32,
                        "type": 3, // u8
                    },
                }
            },
            { // type 7
                "path": [
                    "json",
                    "RecursiveStruct",
                ],
                "def": {
                    "composite": {
                        "fields": [
                            {
                                "name": "rec",
                                "type": 8,
                                "typeName": "Vec<RecursiveStruct>"
                            }
                        ]
                    },
                }
            },
            { // type 8
                "def": {
                    "sequence": {
                        "type": 7, // RecursiveStruct
                    },
                }
            },
            { // type 9
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
            { // type 10
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
                                    { "type": 3, "typeName": "u8" }, // u8
                                    { "type": 4, "typeName": "u32" }, // u32
                                ]
                            },
                            {
                                "name": "C",
                                "fields": [
                                    {
                                        "name": "a",
                                        "type": 3, // u8
                                        "typeName": "u8"
                                    },
                                    {
                                        "name": "b",
                                        "type": 4, // u32
                                        "typeName": "u32"
                                    },
                                    {
                                        "name": "c",
                                        "type": 6,
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
