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
use alloc::{boxed::Box, vec};

use serde::Serialize;
use serde_json::json;
use type_metadata::{
	form::CompactForm, HasTypeDef as _, HasTypeId as _, IntoCompact as _, Metadata, Registry, TypeDef, TypeId,
};

#[derive(Serialize)]
struct TypeIdDef {
	id: TypeId<CompactForm>,
	def: TypeDef<CompactForm>,
}

fn assert_json_for_type<T>(expected_json: serde_json::Value)
where
	T: Metadata,
{
	let mut registry = Registry::new();

	let type_id = T::type_id().into_compact(&mut registry);
	let type_def = T::type_def().into_compact(&mut registry);
	let id_def = TypeIdDef {
		id: type_id,
		def: type_def,
	};

	assert_eq!(serde_json::to_value(id_def).unwrap(), expected_json,);
}

#[test]
fn test_unit_struct() {
	#[derive(Metadata)]
	struct UnitStruct;

	assert_json_for_type::<UnitStruct>(json!({
		"id": {
			"custom.name": 1,
			"custom.namespace": [2],
			"custom.params": [],
		},
		"def": {
			"tuple_struct.types": []
		},
	}));
}

#[test]
fn test_tuple_struct() {
	#[derive(Metadata)]
	struct TupleStruct(i32, [u8; 32], bool);

	assert_json_for_type::<TupleStruct>(json!({
		"id": {
			"custom.name": 1,
			"custom.namespace": [2],
			"custom.params": [],
		},
		"def": {
			"tuple_struct.types": [1, 2, 4]
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
		"id": {
			"custom.name": 1,
			"custom.namespace": [2],
			"custom.params": [],
		},
		"def": {
			"struct.fields": [
				{ "name": 3, "type": 1, },
				{ "name": 4, "type": 2, },
				{ "name": 5, "type": 4, },
			]
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
		"id": {
			"custom.name": 1,
			"custom.namespace": [2],
			"custom.params": [],
		},
		"def": {
			"clike_enum.variants": [
				{ "name": 3, "discriminant": 0, },
				{ "name": 4, "discriminant": 42, },
				{ "name": 5, "discriminant": 2, },
			]
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
		"id": {
			"custom.name": 1,
			"custom.namespace": [2],
			"custom.params": [],
		},
		"def": {
			"enum.variants": [
				{
					"unit_variant.name": 3,
				},
				{
					"tuple_struct_variant.name": 4,
					"tuple_struct_variant.types": [1, 2],
				},
				{
					"struct_variant.name": 5,
					"struct_variant.fields": [
						{ "name": 6, "type": 1, },
						{ "name": 7, "type": 3, },
						{ "name": 8, "type": 5, },
					],
				}
			]
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
		"id": {
			"custom.name": 1,
			"custom.namespace": [2],
			"custom.params": [],
		},
		"def": {
			"union.fields": [
				{ "name": 3, "type": 1, },
				{ "name": 4, "type": 3, },
			]
		},
	}));
}
