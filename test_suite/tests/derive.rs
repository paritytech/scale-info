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

use scale_info::{
	tuple_meta_type, ClikeEnumVariant, EnumVariantStruct, EnumVariantTupleStruct, EnumVariantUnit, HasTypeDef,
	HasTypeId, MetaType, Metadata, NamedField, Namespace, TypeDefClikeEnum, TypeDefEnum, TypeDefStruct,
	TypeDefTupleStruct, TypeDefUnion, TypeId, TypeIdCustom, UnnamedField,
};

fn assert_type_id<T, E>(expected: E)
where
	T: HasTypeId + ?Sized,
	E: Into<TypeId>,
{
	assert_eq!(T::type_id(), expected.into());
}

macro_rules! assert_type_id {
	( $ty:ty, $expected:expr ) => {{
		assert_type_id::<$ty, _>($expected)
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

	let type_id = TypeIdCustom::new("S", Namespace::new(vec!["derive"]).unwrap(), tuple_meta_type!(bool, u8));
	assert_type_id!(S<bool, u8>, type_id.clone());

	let type_def = TypeDefStruct::new(vec![
		NamedField::new("t", bool::meta_type()),
		NamedField::new("u", u8::meta_type()),
	])
	.into();
	assert_eq!(<S<bool, u8>>::type_def(), type_def);

	// With "`Self` typed" fields

	type SelfTyped = S<Box<S<bool, u8>>, bool>;

	let self_typed_id = TypeIdCustom::new(
		"S",
		Namespace::new(vec!["derive"]).unwrap(),
		tuple_meta_type!(Box<S<bool, u8>>, bool),
	);
	assert_type_id!(SelfTyped, self_typed_id);

	assert_eq!(
		SelfTyped::type_def(),
		TypeDefStruct::new(vec![
			NamedField::new("t", <Box<S<bool, u8>>>::meta_type()),
			NamedField::new("u", bool::meta_type()),
		])
		.into(),
	);
}

#[test]
fn tuple_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S<T>(T);

	let type_id = TypeIdCustom::new("S", Namespace::new(vec!["derive"]).unwrap(), tuple_meta_type!(bool));
	assert_type_id!(S<bool>, type_id);

	let type_def = TypeDefTupleStruct::new(vec![UnnamedField::of::<bool>()]).into();
	assert_eq!(<S<bool>>::type_def(), type_def);
}

#[test]
fn unit_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S;

	let type_id = TypeIdCustom::new("S", Namespace::new(vec!["derive"]).unwrap(), vec![]);
	assert_type_id!(S, type_id);

	let type_def = TypeDefTupleStruct::unit().into();
	assert_eq!(S::type_def(), type_def);
}

#[test]
fn c_like_enum_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	enum E {
		A,
		B = 10,
	}

	let type_id = TypeIdCustom::new("E", Namespace::new(vec!["derive"]).unwrap(), vec![]);
	assert_type_id!(E, type_id);

	let type_def = TypeDefClikeEnum::new(vec![
		ClikeEnumVariant::new("A", 0u64),
		ClikeEnumVariant::new("B", 10u64),
	])
	.into();
	assert_eq!(E::type_def(), type_def);
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

	let type_id = TypeIdCustom::new("E", Namespace::new(vec!["derive"]).unwrap(), tuple_meta_type!(bool));
	assert_type_id!(E<bool>, type_id);

	let type_def = TypeDefEnum::new(vec![
		EnumVariantTupleStruct::new("A", vec![UnnamedField::of::<bool>()]).into(),
		EnumVariantStruct::new("B", vec![NamedField::new("b", bool::meta_type())]).into(),
		EnumVariantUnit::new("C").into(),
	])
	.into();
	assert_eq!(<E<bool>>::type_def(), type_def);
}

#[test]
// #[should_panic] // TODO: remove #[should_panic]
fn union_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	union U<T: Copy> {
		u: T,
	}

	let type_id = TypeIdCustom::new("U", Namespace::new(vec!["derive"]).unwrap(), tuple_meta_type!(bool));
	assert_type_id!(U<bool>, type_id);

	let type_def = TypeDefUnion::new(vec![NamedField::new("u", bool::meta_type())]).into();
	assert_eq!(<U<bool>>::type_def(), type_def);
}
