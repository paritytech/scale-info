// Copyright 2019-2020
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

#![allow(unused)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use assert_json_diff::assert_json_eq;
use scale_info::{form::CompactForm, IntoCompact as _, Metadata, Registry};
use serde_json::json;

fn assert_json_for_type<T>(expected_json: serde_json::Value)
where
	T: Metadata,
{
	let mut registry = Registry::new();

	let ty = T::type_info().into_compact(&mut registry);

	assert_json_eq!(serde_json::to_value(ty).unwrap(), expected_json,);
}

#[test]
fn test_unit_struct() {
	#[derive(Metadata)]
	struct UnitStruct;

	assert_json_for_type::<UnitStruct>(json!({
		"composite": {
			"path": [1, 2]
		},
	}));
}

#[test]
fn test_tuplestruct() {
	#[derive(Metadata)]
	struct TupleStruct(i32, [u8; 32], bool);

	assert_json_for_type::<TupleStruct>(json!({
		"composite": {
			"path": [1, 2],
			"fields": [
				{ "type": 1 },
				{ "type": 2 },
				{ "type": 4 },
			],
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
		"composite": {
			"path": [1, 2],
			"fields": [
				{ "name": 3, "type": 1, },
				{ "name": 4, "type": 2, },
				{ "name": 5, "type": 4, },
			],
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
		"variant": {
			"path": [1, 2],
			"variants": [
				{ "name": 3, "discriminant": 0, },
				{ "name": 4, "discriminant": 42, },
				{ "name": 5, "discriminant": 2, },
			],
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
		"variant": {
			"path": [1, 2],
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
	}));
}

#[test]
fn test_associated_types() {
	trait A {
		type B;
	}

	#[derive(Metadata)]
	struct C<T>
	where
		T: A,
	{
		a: T::B, // idea: could infer assoc types from usage
	}

	struct D {}

	impl A for D {
		type B = bool;
	}

	let mut registry = Registry::new();
	registry.register_type(&C::<D>::meta_type());

	let expected_json = json!({
		"strings": [
			"json",      	   //  1
			"A",      		//  2

		],
		"types": [
		]
	});
}

#[test]
fn test_generics() {
	let mut registry = Registry::new();

	#[derive(Metadata)]
	struct GenericStruct<T> {
		a: T, // Should look up in the set of all type params for a matching parameter (use any::TypeId?) Field::of_parameter::<T>()
		b: Option<T>, // Field::of_parameterized::<Option<T>>(parameters!(param(T));
		c: Option<bool>, // Should point to non parameterized type Field::of::<Option<bool>>(): TypeId::Any
		      // d: (Option<T>, Option<bool>), // Field::of_parameterized::<Option<T>>(parameters!(param(T), concrete(bool)) // left to right params (scope stack)
		      // d: Option<GenericStruct<T, bool>>,
		      // e: Vec<(U, Option<T>)>, // Should resolve to correct parameters
		      // f: Result<
		      // 	GenericStruct<T, bool>, // same type as nested in field d
		      //  ()
		      // >
	}

	// The set of type parameters here is [T]
	// The challenge is to make sure the params line up e.g. if the set is [T,U]
	// make sure that the params get substituted in the right places
	// Should be able to do this with concrete `any::TypeId`

	// Solution to the parameter matching, specify which types are parameterized, and which concrete
	// e.g.

	#[derive(Metadata)]
	struct ConcreteStruct {
		a: GenericStruct<bool>,
		b: Option<u32>,
		c: GenericStruct<Option<bool>>,
	}

	assert_json_for_type::<ConcreteStruct>(json!(
		"strings": [
			"json",      		//  1
			"GenericStruct",   	//  2
			"T",				//  3
			"Option",		   	//  4
			"Some",		   		//  5
			"None",		   		//  6
			"ConcreteStruct",  	//  7
			"a",               	//  8
			"b",               	//  9
			"c",               	//  10
		],
		"types": [
			{ // type 1
				"definition": {
					"ty": {
						"primitive": "bool"
					}
				}
			},
			{ // type 2
				"definition": {
					"ty": {
						"primitive": "u32"
					}
				}
			},
			{ // type 3
				// T
				"parameter": {
					"path": [4], 	// Option
					"name": 3, 		// T
				}
			},
			{ // type 4
				// Option<T>
				"definition": {
					"path": [4], 	// Option
					"params": [3], 	// Option::T
					"ty": {
						"variant": {
							"variants": [
								{ // Some(T)
									"name": 4,
									"fields": [
										{ "type": 3 }, // Option::T
									],
								},
								{ // None
									"name": 5,
								}
							]
						}
					}
				}
			},
			{ // type 5
				// GenericStruct::T
				"parameter": {
					"type": [2], 	// GenericStruct
					"name": 3, 		// T
				}
			},
			{ // type 6
				// Option<GenericStruct::T>
				"generic": {
					"type": 4,		// Option<T>
					"params": [5]	// GenericStruct::T
				}
			},
			{ // type 5
				// GenericStruct<T>
				"definition": {
					"path": [1, 2],
					"params": [5], // GenericStruct::T
					"ty": {
						"composite": {
							"fields": [
								{ "name": 7, "type": 3 } 	// a: GenericStruct::T
								{ "name": 8, "type": 6 } 	// b: Option<GenericStruct::T>
								{ "name": 9, "type": 10 } 	// c: Option<bool>
								{ "name": 10, "type": 9 } 	// d: Option<GenericStruct<T>>,
							]
						}
					}
				}
			},
			{ // type 6
				// GenericStruct<bool>
				"generic": {
					"type": 5,		// GenericStruct<T>
					"params": [1]	// bool
				}
			},
			{ // type 7
				// Option<u32>
				"generic": {
					"type": 4,		// Option<T>
					"params": [2]	// u32
				}
			},
			{ // type 8
				// GenericStruct<GenericStruct::T>
				"generic": {
					"type": 5,		// GenericStruct<T>
					"params": [5]	// GenericStruct::T
				}
			},
			{ // type 9
				// Option<GenericStruct<GenericStruct::T>>
				"generic": {
					"type": 4,		// Option<T>
					"params": [8]	// GenericStruct<GenericStruct::T>
				}
			},
			{ // type 10
				// Option<bool>
				"generic": {
					"type": 4,		// Option<T>
					"params": [1]	// bool
				}
			},
			{ // type 11
				// ConcreteStruct
				"definition": {
					"path": [1, 2],
					"ty": {
						"composite": {
							"path": [1, 6],
							"fields": [
								{ "name": 8, "type": 5 } // a: GenericStruct<bool>
								{ "name": 9, "type": 7 } // b: Option<u32>
								{ "name": 10, "type": 9 } // c: Vec<Option<T>>
							]
						}
					}
				}
			},
		]
	))
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
				"composite": {
					"path": [
						1, // json
						2, // UnitStruct
					]
				},
			},
			{ // type 2
				"composite": {
					"path": [
						1, // json
						3, // TupleStruct
					],
					"fields": [
						{ "type": 3 },
						{ "type": 4 },
					],
				},
			},
			{ // type 3
				"primitive": "u8",
			},
			{ // type 4
				"primitive": "u32",
			},
			{ // type 5
				"composite": {
					"path": [
						1, // json
						4, // Struct
					],
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
			},
			{ // type 6
				"array": {
					"len": 32,
					"type": 3, // u8
				},
			},
			{ // type 7
				"composite": {
					"path": [
						1, // json
						8, // RecursiveStruct
					],
					"fields": [
						{
							"name": 9, // rec
							"type": 8, // Vec<RecursiveStruct>
						}
					]
				},
			},
			{ // type 8
				"sequence": {
					"type": 7, // RecursiveStruct
				},
			},
			{ // type 9
				"variant": {
					"path": [
						1, 	// json
						10, // CLikeEnum
					],
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
			},
			{ // type 10
				"variant": {
					"path": [
						1, 	// json
						14, // RustEnum
					],
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
			},
		]
	});

	assert_json_eq!(serde_json::to_value(registry).unwrap(), expected_json,);
}
