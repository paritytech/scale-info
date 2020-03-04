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

use type_metadata::{
	product_type, sum_type, tuple_meta_type, ClikeEnumVariant, EnumVariantStruct, EnumVariantTupleStruct,
	EnumVariantUnit, TypeInfo, Metadata, NamedField, Namespace, Type, TypeProductStruct, TypeProductTupleStruct,
	TypeSumClikeEnum, TypeSumEnum, UnnamedField,
};

fn assert_type<T, E>(expected: E)
where
	T: TypeInfo + ?Sized,
	E: Into<Type>,
{
	assert_eq!(T::type_info(), expected.into());
}

macro_rules! assert_type {
	( $ty:ty, $expected:expr ) => {{
		assert_type::<$ty, _>($expected)
		}};
}

#[test]
fn struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S<T, U> {
		pub t: T,
		pub u: U,
	}

	let struct_type = product_type(
		"S",
		Namespace::new(vec!["derive"]).unwrap(),
		tuple_meta_type!(bool, u8),
		TypeProductStruct::new(vec![
			NamedField::new("t", bool::meta_type()),
			NamedField::new("u", u8::meta_type()),
		]),
	);
	assert_type!(S<bool, u8>, struct_type.clone());

	// With "`Self` typed" fields

	type SelfTyped = S<Box<S<bool, u8>>, bool>;

	let self_typed_type = product_type(
		"S",
		Namespace::new(vec!["derive"]).unwrap(),
		tuple_meta_type!(Box<S<bool, u8>>, bool),
		TypeProductStruct::new(vec![
			NamedField::new("t", <Box<S<bool, u8>>>::meta_type()),
			NamedField::new("u", bool::meta_type()),
		]),
	);
	assert_type!(SelfTyped, self_typed_type);
}

#[test]
fn tuple_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S<T>(T);

	let ty = product_type(
		"S",
		Namespace::new(vec!["derive"]).unwrap(),
		tuple_meta_type!(bool),
		TypeProductTupleStruct::new(vec![UnnamedField::of::<bool>()]),
	);
	assert_type!(S<bool>, ty);
}

#[test]
fn unit_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S;

	let ty = product_type(
		"S",
		Namespace::new(vec!["derive"]).unwrap(),
		vec![],
		TypeProductTupleStruct::unit(),
	);

	assert_type!(S, ty);
}

#[test]
fn c_like_enum_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	enum E {
		A,
		B = 10,
	}

	let ty = sum_type(
		"E",
		Namespace::new(vec!["derive"]).unwrap(),
		vec![],
		TypeSumClikeEnum::new(vec![
			ClikeEnumVariant::new("A", 0u64),
			ClikeEnumVariant::new("B", 10u64),
		]),
	);

	assert_type!(E, ty);
}

#[test]
fn enum_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	enum E<T> {
		A(T),
		B { b: T },
		C,
	}

	let ty = sum_type(
		"E",
		Namespace::new(vec!["derive"]).unwrap(),
		tuple_meta_type!(bool),
		TypeSumEnum::new(vec![
			EnumVariantTupleStruct::new("A", vec![UnnamedField::of::<bool>()]).into(),
			EnumVariantStruct::new("B", vec![NamedField::new("b", bool::meta_type())]).into(),
			EnumVariantUnit::new("C").into(),
		]),
	);

	assert_type!(E<bool>, ty);
}
