// Copyright 2019
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use assert_json_diff::assert_json_eq;
use serde::Serialize;
use serde_json::json;
use type_metadata::{form::CompactForm, IntoCompact as _, Metadata, Registry, TypeDef, Type};

fn assert_json_for_type<T>(expected_json: serde_json::Value)
where
	T: Metadata,
{
	let mut registry = Registry::new();

	let type_id = T::get_type().into_compact(&mut registry);

	assert_json_eq!(serde_json::to_value(type_id).unwrap(), expected_json,);
}

#[test]
fn test_unit_struct() {
	#[derive(Metadata)]
	struct UnitStruct;

	assert_json_for_type::<UnitStruct>(json!({
		"custom": {
			"name": 1,
			"namespace": [2],
			"params": [],
			"def": {
				"tuplestruct": {
					"types": []
				}
			},
		},
	}));
}

#[test]
fn test_tuplestruct() {
	#[derive(Metadata)]
	struct TupleStruct(i32, [u8; 32], bool);

	assert_json_for_type::<TupleStruct>(json!({
		"custom": {
			"name": 1,
			"namespace": [2],
			"params": [],
			"def": {
				"tuplestruct": {
					"types": [1, 2, 4]
				}
			},
		},
	}));
}

#[test]
fn test_struct() {
	#[derive(Metadata)]
	struct Struct {
		a: i32,
		b: [u8; 32],
		c: bool,
	}

	assert_json_for_type::<Struct>(json!({
		"custom": {
			"name": 1,
			"namespace": [2],
			"params": [],
			"def": {
				"struct": {
					"fields": [
						{ "name": 3, "type": 1, },
						{ "name": 4, "type": 2, },
						{ "name": 5, "type": 4, },
					]
				}
			},
		},
	}));
}

#[test]
fn test_clike_enum() {
	#[derive(Metadata)]
	enum ClikeEnum {
		A,
		B = 42,
		C,
	}

	assert_json_for_type::<ClikeEnum>(json!({
		"custom": {
			"name": 1,
			"namespace": [2],
			"params": [],
			"def": {
				"clikeenum": {
					"variants": [
						{ "name": 3, "discriminant": 0, },
						{ "name": 4, "discriminant": 42, },
						{ "name": 5, "discriminant": 2, },
					]
				}
			},
		},
	}));
}

#[test]
fn test_enum() {
	#[derive(Metadata)]
	enum Enum {
		ClikeVariant,
		TupleStructVariant(u32, bool),
		StructVariant { a: u32, b: [u8; 32], c: char },
	}

	assert_json_for_type::<Enum>(json!({
		"custom": {
			"name": 1,
			"namespace": [2],
			"params": [],
			"def": {
				"enum": {
					"variants": [
						{
							"unit": {
								"name": 3,
							}
						},
						{
							"tuplestruct": {
								"name": 4,
								"types": [1, 2],
							}
						},
						{
							"struct": {
								"name": 5,
								"fields": [
									{ "name": 6, "type": 1, },
									{ "name": 7, "type": 3, },
									{ "name": 8, "type": 5, },
								],
							}
						}
					]
				}
			},
		},
	}));
}

#[test]
fn test_union() {
	#[derive(Metadata)]
	union Union {
		inl: [u8; 32],
		ext: u128,
	}

	assert_json_for_type::<Union>(json!({
		"custom": {
			"name": 1,
			"namespace": [2],
			"params": [],
			"def": {
				"union": {
					"fields": [
						{ "name": 3, "type": 1, },
						{ "name": 4, "type": 3, },
					]
				}
			},
		},
	}));
}

#[test]
fn test_registry() {
	let mut registry = Registry::new();

	#[derive(Metadata)]
	struct UnitStruct;
	#[derive(Metadata)]
	struct TupleStruct(u8, u32);
	#[derive(Metadata)]
	struct Struct {
		a: u8,
		b: u32,
		c: [u8; 32],
	}
	#[derive(Metadata)]
	struct RecursiveStruct {
		rec: Vec<RecursiveStruct>,
	}
	#[derive(Metadata)]
	enum ClikeEnum {
		A,
		B,
		C,
	}
	#[derive(Metadata)]
	enum RustEnum {
		A,
		B(u8, u32),
		C { a: u8, b: u32, c: [u8; 32] },
	}

	registry.register_type(&UnitStruct::meta_type());
	registry.register_type(&TupleStruct::meta_type());
	registry.register_type(&Struct::meta_type());
	registry.register_type(&RecursiveStruct::meta_type());
	registry.register_type(&ClikeEnum::meta_type());
	registry.register_type(&RustEnum::meta_type());

	let expected_json = json!({
		"strings": [
			"UnitStruct",      //  1
			"json",            //  2
			"TupleStruct",     //  3
			"Struct",          //  4
			"a",               //  5
			"b",               //  6
			"c",               //  7
			"RecursiveStruct", //  8
			"rec",             //  9
			"Vec",             // 10
			"ClikeEnum",       // 11
			"A",               // 12
			"B",               // 13
			"C",               // 14
			"RustEnum",        // 15
		],
		"types": [
			{ // type 1
				"custom": {
					"name": 1, // UnitStruct
					"namespace": [2], // json
					"params": [],
					"def": {
						"tuplestruct": {
							"types": [],
						}
					},
				},
			},
			{ // type 2
				"custom": {
					"name": 3, // TupleStruct
					"namespace": [2], // json
					"params": [],
					"def": {
						"tuplestruct": {
							"types": [
								3, // u8
								4, // u32
							]
						}
					}
				},
			},
			{ // type 3
				"primitive": "u8",
			},
			{ // type 4
				"primitive": "u32",
			},
			{ // type 5
				"custom": {
					"name": 4, // Struct
					"namespace": [2], // json
					"params": [],
					"def": {
						"struct": {
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
					}
				},
			},
			{ // type 6
				"array": {
					"len": 32,
					"type": 3, // u8
				},
			},
			{ // type 7
				"custom": {
					"name": 8, // RecursiveStruct
					"namespace": [2], // json
					"params": [],
					"def": {
						"struct": {
							"fields": [
								{
									"name": 9, // rec
									"type": 8, // Vec<RecursiveStruct>
								}
							]
						}
					}
				},
			},
			{ // type 8
				"collection": {
					"name": 10, // Vec
					"type": 7,
				},
			},
			{ // type 9
				"custom": {
					"name": 11, // ClikeEnum
					"namespace": [2], // json
					"params": [],
					"def": {
						"clikeenum": {
							"variants": [
								{
									"name": 12, // A
									"discriminant": 0,
								},
								{
									"name": 13, // B
									"discriminant": 1,
								},
								{
									"name": 14, // C
									"discriminant": 2,
								},
							]
						}
					}
				},
			},
			{ // type 10
				"custom": {
					"name": 15, // RustEnum
					"namespace": [2], // json
					"params": [],
					"def": {
						"enum": {
							"variants": [
								{
									"unit": {
										"name": 12,
									} // A
								},
								{
									"tuplestruct": {
										"name": 13, // B
										"types": [
											3, // u8
											4, // u32
										]
									},

								},
								{
									"struct": {
										"name": 14, // C
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
								}
							]
						}
					}
				},
			},
		]
	});

	assert_json_eq!(serde_json::to_value(registry).unwrap(), expected_json,);
}
