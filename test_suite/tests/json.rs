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

use pretty_assertions::{assert_eq, assert_ne};
use scale_info::{form::CompactForm, IntoCompact as _, Metadata, Registry};
use serde_json::json;

fn assert_json_for_type<T>(expected_json: serde_json::Value)
where
	T: Metadata,
{
	let mut registry = Registry::new();

	let ty = T::type_info().into_compact(&mut registry);

	// panic!(&serde_json::to_string_pretty(&registry).unwrap());
	assert_eq!(expected_json, serde_json::to_value(ty).unwrap());
}

#[test]
fn test_unit_struct() {
	#[derive(Metadata)]
	struct UnitStruct;

	assert_json_for_type::<UnitStruct>(json!({
		"composite": { },
	}));
}

#[test]
fn test_tuplestruct() {
	#[derive(Metadata)]
	struct TupleStruct(i32, [u8; 32], bool);

	assert_json_for_type::<TupleStruct>(json!({
		"composite": {
			"fields": [
				{ "type": 1 },
				{ "type": 4 },
				{ "type": 5 },
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
			"fields": [
				{ "name": 1, "type": 1, },
				{ "name": 2, "type": 4, },
				{ "name": 3, "type": 5, },
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
			"variants": [
				{ "name": 1, "discriminant": 0, },
				{ "name": 2, "discriminant": 42, },
				{ "name": 3, "discriminant": 2, },
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
			"variants": [
				{ "name": 1 },
				{
					"name": 2,
					"fields": [
						{ "type": 1 },
						{ "type": 2 },
					],
				},
				{
					"name": 3,
					"fields": [
						{ "name": 4, "type": 1, },
						{ "name": 5, "type": 5, },
						{ "name": 6, "type": 6, },
					],
				}
			],
		},
	}));
}

// #[test]
// fn test_associated_types() {
// 	trait A {
// 		type B;
// 	}
//
// 	#[derive(Metadata)]
// 	struct C<T>
// 	where
// 		T: A,
// 	{
// 		a: T::B, // idea: could infer assoc types from usage
// 	}
//
// 	struct D {}
//
// 	impl A for D {
// 		type B = bool;
// 	}
//
// 	let mut registry = Registry::new();
// 	registry.register_type(&C::<D>::meta_type());
//
// 	let expected_json = json!({
// 		"strings": [
// 			"json",      	   //  1
// 			"A",      		//  2
//
// 		],
// 		"types": [
// 		]
// 	});
// }

#[test]
fn test_generics() {
	let mut registry = Registry::new();

	#[derive(Metadata)]
	struct GenericStruct<T> {
		a: T,
		b: Option<T>,
		c: Option<bool>,
		// d: (Option<T>, Option<bool>), // Field::of_parameterized::<Option<T>>(parameters!(param(T), concrete(bool)) // left to right params (scope stack)
		// d: Option<GenericStruct<T, bool>>,
		// e: Vec<(U, Option<T>)>, // Should resolve to correct parameters
		// f: Result<
		// 	GenericStruct<T, bool>, // same type as nested in field d
		//  ()
		// >
	}

	#[derive(Metadata)]
	struct ConcreteStruct {
		a: GenericStruct<bool>,
		b: Option<u32>,
		c: GenericStruct<Option<bool>>,
	}

	registry.register_type(&ConcreteStruct::meta_type());

	let expected_json = json!({
		"strings": [
			"json",				// 1
			"ConcreteStruct",	// 2
			"a",				// 3
			"GenericStruct",	// 4
			"T",				// 5
			"b",				// 6
			"Option",			// 7
			"None",				// 8
			"Some",				// 9
			"c"					// 10
		],
		"types": [
			{ // type 1
				"definition": {
					"path": [
						1,	// json
						2	// ConcreteStruct
					],
					"ty": {
						"composite": {
							"fields": [
								{
									"name": 3,	// a
									"type": 9	// GenericStruct<Bool>
								},
								{
									"name": 6, 	// b
									"type": 11	// Option<u32>
								},
								{
									"name": 10, // c
									"type": 12	// GenericStruct<Option<bool>>
								}
							]
						}
					}
				}
			},
			{ // type 2: bool
				"definition": {
					"ty": {
						"primitive": "bool"
					}
				}
			},
			{ // type 3: GenericStruct<T>
				"definition": {
					"path": [
						1, 	// json
						4	// GenericStruct
					],
					"ty": {
						"composite": {
							"fields": [
								{
									"name": 3,	// a
									"type": 4	// Param T of GenericStruct<T>
								},
								{
									"name": 6,	// b
									"type": 7 	// Option<T> where T is Param T of GenericStruct<T>
								},
								{
									"name": 10, // c
									"type": 8	// Option<bool>
								}
							]
						}
					}
				}
			},
			{ // type 4: Param T of GenericStruct<T>
				"parameter": {
					"name": 5,	// T
					"parent": 3	// GenericStruct<T>
				}
			},
			{ // type 5: Option<T>
				"definition": {
					"path": [
						7	// Option
					],
					"ty": {
						"variant": {
							"variants": [
								{
									"name": 8 	// None
								},
								{
									"name": 9,	// Some
									"fields": [
										{
											"type": 6 // Param T of Option<T>
										}
									]
								}
							]
						}
					}
				}
			},
			{ // type 6: Param T of Option<T>
				"parameter": {
					"name": 5,	// T
					"parent": 5	// Option<T>
				}
			},
			{ // type 7: Option<T> where T is Param T of GenericStruct<T>
				"generic": {
					"ty": 5,	// Option<T>
					"params": [
						4		// Param T of GenericStruct<T>
					]
				}
			},
			{ // type 8: Option<bool>
				"generic": {
					"ty": 5,	// Option<T>
					"params": [
						2		// bool
					]
				}
			},
			{ // type 9: GenericStruct<bool>
				"generic": {
					"ty": 3,	// GenericStruct<T>
					"params": [
						2		// bool
					]
				}
			},
			{ // type 10: u32
				"definition": {
					"ty": {
						"primitive": "u32"
					}
				}
			},
			{ // type 11: Option<u32>
				"generic": {
					"ty": 5,	// Option<T>
					"params": [
						10		// u32
					]
				}
			},
			{ // type 12: GenericStruct<Option<bool>>
				"generic": {
					"ty": 3,	// GenericStruct<T>
					"params": [
						8		// Option<bool>
					]
				}
			}
		]
	});

	assert_eq!(expected_json, serde_json::to_value(registry).unwrap());
}

#[test]
fn test_generic_parameters_used_out_of_order() {
	let mut registry = Registry::new();

	#[derive(Metadata)]
	struct GenericStruct<T, U> {
		a: U,
		b: T,
	}

	registry.register_type(&GenericStruct::<u8, u16>::meta_type());

	let expected_json = json!({
		"strings": [
			"json",				// 1
			"GenericStruct",	// 2
			"a",				// 3
			"U",				// 4
			"b",				// 5
			"T",				// 6
		],
		"types": [
			{ // type 1: u8
				"definition": {
					"ty": {
						"primitive": "u8"
					}
				}
			},{ // type 2: u16
				"definition": {
					"ty": {
						"primitive": "u16"
					}
				}
			},
			{ // type 3: GenericStruct<T, U>
				"definition": {
					"path": [
						1, 	// json
						2	// GenericStruct
					],
					"ty": {
						"composite": {
							"fields": [
								{
									"name": 3,	// a
									"type": 4	// Param U of GenericStruct<T, U>
								},
								{
									"name": 5,	// b
									"type": 5 	// Option<T> where T is Param T of GenericStruct<T, U>
								},
							]
						}
					}
				}
			},
			{ // type 4: Param U of GenericStruct<T, U>
				"parameter": {
					"name": 4,	// U
					"parent": 3	// GenericStruct<T, U>
				}
			},
			{ // type 5: Param T of GenericStruct<T, U>
				"parameter": {
					"name": 6,	// T
					"parent": 3	// GenericStruct<T, U>
				}
			},
			{ // type 6: GenericStruct<u8, u16>
				"generic": {
					"ty": 3,		// GenericStruct<T, U>
					"params": [
						1,			// u8
						2, 			// u16
					]
				}
			}
		]
	});
	assert_eq!(expected_json, serde_json::to_value(registry).unwrap());
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
			"Sequence",        //  10
			"ClikeEnum",       //  11
			"A",               //  12
			"B",               //  13
			"C",               //  14
			"RustEnum",        //  15
		],
		"types": [
			{ // type 1
				"definition": {
					"path": [
						1, // json
						2, // UnitStruct
					],
					"ty": {
						"composite": { },
					}
				}
			},
			{ // type 2
				"definition": {
					"path": [
						1, // json
						3, // TupleStruct
					],
					"ty": {
						"composite": {
							"fields": [
								{ "type": 3 },
								{ "type": 4 },
							],
						},
					}
				}
			},
			{ // type 3
				"definition": {
					"ty": {
						"primitive": "u8",
					}
				}
			},
			{ // type 4
				"definition": {
					"ty": {
						"primitive": "u32",
					}
				}			},
			{ // type 5
				"definition": {
					"path": [
						1, // json
						4, // Struct
					],
					"ty": {
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
									"type": 7, // [u8; 32]
								}
							]
						},
					}
				}
			},
			{ // type 6
				"definition": {
					"ty": {
						"array": {
							"len": 32,
							"type": 3, // u8 // todo: should be generic param T
						},
					}
				}
			},
			{ // type 7
				"generic": {
					"ty": 6,	// [T; 32]
					"params": [
						3	 	// u8
					]
				}
			},
			{ // type 8
				"definition": {
					"path": [
						1, // json
						8, // RecursiveStruct
					],
					"ty": {
							"composite": {
							"fields": [
								{
									"name": 9, // rec
									"type": 10, // Vec<RecursiveStruct>
								}
							]
						},
					}
				}
			},
			{ // type 9
				"definition": {
					"path": [
						10	// Sequence
					],
					"ty": {
						"sequence": {
							"type": 8, // RecursiveStruct
						},
					}
				}
			},
			{ // type 10: Vec<RecursiveStruct>
				"generic": {
					"ty": 9,	// Vec<T>
					"params": [
						8		// RecursiveStruct
					]
				}
			},
			{ // type 11
				"definition": {
					"path": [
						1, 	// json
						11, // CLikeEnum
					],
					"ty": {
						"variant": {
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
				}

			},
			{ // type 12
				"definition": {
					"path": [
						1, 	// json
						15, // RustEnum
					],
					"ty": {
						"variant": {
							"variants": [
								{
									"name": 12, // A
								},
								{
									"name": 13, // B
									"fields": [
										{ "type": 3 }, // u8
										{ "type": 4 }, // u32
									]
								},
								{
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
											"type": 7, // [u8; 32]
										}
									]
								}
							]
						},
					}
				}
			},
		]
	});

	assert_eq!(expected_json, serde_json::to_value(registry).unwrap());
}
