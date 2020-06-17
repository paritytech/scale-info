// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use pretty_assertions::{assert_eq, assert_ne};
use scale_info::{form::CompactForm, IntoCompact as _, TypeInfo, Registry, meta_type};
use serde_json::json;

fn assert_json_for_type<T>(expected_json: serde_json::Value)
where
	T: TypeInfo + ?Sized,
{
	let mut registry = Registry::new();

	let ty = T::type_info().into_compact(&mut registry);

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
	assert_json_for_type::<[u8; 2]>(json!({ "def": { "array": { "len": 2, "type": 1 } } }));
	assert_json_for_type::<[bool; 4]>(json!({ "def": { "array": { "len": 4, "type": 1 } } }));
	assert_json_for_type::<[char; 8]>(json!({ "def": { "array": { "len": 8, "type": 1 } } }));
	// tuples
	assert_json_for_type::<(u8, bool)>(json!({ "def": { "tuple": [ 1, 2 ] } }));
	assert_json_for_type::<(u8, bool, char, u128)>(json!({ "def": { "tuple": [ 1, 2, 3, 4 ] } }));
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
		"path": [1],
		"params": [1],
		"def": {
			"variant": {
				"variants": [
					{
						"name": 2,
					},
					{
						"name": 3,
						"fields": [ { "type": 1 } ]
					},
				]
			}
		}
	}));
	assert_json_for_type::<Result<u32, u64>>(json!({
		"path": [1],
		"params": [1, 2],
		"def": {
			"variant": {
				"variants": [
					{
						"name": 2,
						"fields": [ { "type": 1 } ]
					},
					{
						"name": 3,
						"fields": [ { "type": 2 } ]
					}
				]
			}
		}
	}));
	// references
	assert_json_for_type::<&bool>(json!({ "def": { "primitive": "bool" } }));
	assert_json_for_type::<&mut str>(json!({ "def": { "primitive": "str" } }));
	assert_json_for_type::<alloc::boxed::Box<u32>>(json!({ "def": { "primitive": "u32" } }));
	// strings
	assert_json_for_type::<alloc::string::String>(json!({ "def": { "primitive": "str" } }));
	assert_json_for_type::<str>(json!({ "def": { "primitive": "str" } }));
	// PhantomData
	assert_json_for_type::<core::marker::PhantomData<bool>>(json!({
		"path": [1],
		"params": [1],
		"def": {
			"composite": {},
		}
	}))
}

#[test]
fn test_unit_struct() {
	#[derive(TypeInfo)]
	struct UnitStruct;

	assert_json_for_type::<UnitStruct>(json!({
		"path": [1, 2],
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
		"path": [1, 2],
		"def": {
			"composite": {
				"fields": [
					{ "type": 1 },
					{ "type": 2 },
					{ "type": 4 },
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
		"path": [1, 2],
		"def": {

		},
		"def": {
			"composite": {
				"fields": [
					{ "name": 3, "type": 1, },
					{ "name": 4, "type": 2, },
					{ "name": 5, "type": 4, },
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
		"path": [1, 2],
		"def": {
			"variant": {
				"variants": [
					{ "name": 3, "discriminant": 0, },
					{ "name": 4, "discriminant": 42, },
					{ "name": 5, "discriminant": 2, },
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
		"path": [1, 2],
		"def": {
			"variant": {
				"variants": [
					{ "name": 3 },
					{
						"name": 4,
						"fields": [
							{ "type": 1 },
							{ "type": 2 },
						],
					},
					{
						"name": 5,
						"fields": [
							{ "name": 6, "type": 1, },
							{ "name": 7, "type": 3, },
							{ "name": 8, "type": 5, },
						],
					}
				],
			},
		}
	}));
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
		"strings": [
			"json",      	   //  1
			"UnitStruct",      //  2
			"TupleStruct",     //  3
			"Struct",          //  4
			"a",               //  5
			"b",               //  6
			"c",               //  7
			"RecursiveStruct", //  8
			"rec",             //  9
			"ClikeEnum",       // 10
			"A",               // 11
			"B",               // 12
			"C",               // 13
			"RustEnum",        // 14
		],
		"types": [
			{ // type 1
				"path": [
					1, // json
					2, // UnitStruct
				],
				"def": {
					"composite": {},
				}
			},
			{ // type 2
				"path": [
					1, // json
					3, // TupleStruct
				],
				"def": {
					"composite": {
						"fields": [
							{ "type": 3 },
							{ "type": 4 },
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
					1, // json
					4, // Struct
				],
				"def": {
					"composite": {
						"fields": [
							{
								"name": 5, // a
								"type": 3, // u8
							},
							{
								"name": 6, // b
								"type": 4, // u32
							},
							{
								"name": 7, // c
								"type": 6, // [u8; 32]
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
					1, // json
					8, // RecursiveStruct
				],
				"def": {
					"composite": {
						"fields": [
							{
								"name": 9, // rec
								"type": 8, // Vec<RecursiveStruct>
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
					1, 	// json
					10, // CLikeEnum
				],
				"def": {
					"variant": {
						"variants": [
							{
								"name": 11, // A
								"discriminant": 0,
							},
							{
								"name": 12, // B
								"discriminant": 1,
							},
							{
								"name": 13, // C
								"discriminant": 2,
							},
						]
					}
				}
			},
			{ // type 10
				"path": [
					1, 	// json
					14, // RustEnum
				],
				"def": {
					"variant": {
						"variants": [
							{
								"name": 11, // A
							},
							{
								"name": 12, // B
								"fields": [
									{ "type": 3 }, // u8
									{ "type": 4 }, // u32
								]
							},
							{
								"name": 13, // C
								"fields": [
									{
										"name": 5, // a
										"type": 3, // u8
									},
									{
										"name": 6, // b
										"type": 4, // u32
									},
									{
										"name": 7, // c
										"type": 6, // [u8; 32]
									}
								]
							}
						]
					},
				}
			},
		]
	});

	assert_eq!(serde_json::to_value(registry).unwrap(), expected_json,);
}
