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

use crate::*;
use core::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec};

fn assert_type_id<T, E>(expected: E)
where
	T: HasType + ?Sized,
	E: Into<Type>,
{
	assert_eq!(T::type_id(), expected.into());
}

macro_rules! assert_type_id {
	( $ty:ty, $expected:expr ) => {{
		assert_type_id::<$ty, _>($expected)
		}};
}

#[test]
fn primitives() {
	assert_type_id!(bool, TypePrimitive::Bool);
	assert_type_id!(&str, TypePrimitive::Str);
	assert_type_id!(i8, TypePrimitive::I8);

	assert_type_id!([bool], TypeSlice::new(bool::meta_type()));
}

#[test]
fn prelude_items() {
	assert_type_id!(
		String,
		TypeCustom::new(
			"String",
			Namespace::prelude(),
			Vec::new(),
			TypeDefStruct::new(vec![NamedField::new("vec", MetaType::new::<Vec<u8>>())]).into(),
		)
	);

	assert_type_id!(
		Option<u128>,
		TypeCustom::new(
			"Option",
			Namespace::prelude(),
			tuple_meta_type!(u128),
			TypeDefEnum::new(vec![
				EnumVariantUnit::new("None").into(),
				EnumVariantTupleStruct::new("Some", vec![UnnamedField::of::<u128>()]).into(),
			])
			.into(),
		)
	);
	assert_type_id!(
		Result<bool, String>,
		TypeCustom::new(
			"Result",
			Namespace::prelude(),
			tuple_meta_type!(bool, String),
			TypeDefEnum::new(vec![
				EnumVariantTupleStruct::new("Ok", vec![UnnamedField::of::<bool>()]).into(),
				EnumVariantTupleStruct::new("Err", vec![UnnamedField::of::<String>()]).into(),
			]).into(),
		)
	);
	assert_type_id!(
		PhantomData<i32>,
		TypeCustom::new(
			"PhantomData",
			Namespace::prelude(),
			tuple_meta_type!(i32),
			TypeDefTupleStruct::new(vec![]).into(),
		)
	)
}

#[test]
fn tuple_primitives() {
	// unit
	assert_type_id!((), TypeTuple::new(tuple_meta_type!()));

	// tuple with one element
	assert_type_id!((bool,), TypeTuple::new(tuple_meta_type!(bool)));

	// tuple with multiple elements
	assert_type_id!((bool, String), TypeTuple::new(tuple_meta_type!(bool, String)));

	// nested tuple
	assert_type_id!(
		((i8, i16), (u32, u64)),
		TypeTuple::new(vec![<(i8, i16)>::meta_type(), <(u32, u64)>::meta_type(),])
	);
}

#[test]
fn array_primitives() {
	// array
	assert_type_id!([bool; 3], TypeArray::new(3, bool::meta_type()));
	// nested
	assert_type_id!([[i32; 5]; 5], TypeArray::new(5, <[i32; 5]>::meta_type()));
	// slice
	assert_type_id!([bool], TypeSlice::new(bool::meta_type()));
	// vec
	assert_type_id!(Vec<bool>, TypeCollection::of::<bool>("Vec"));
}

#[test]
fn struct_with_generics() {
	#[allow(unused)]
	struct MyStruct<T> {
		data: T,
	}

	impl<T> HasType for MyStruct<T>
	where
		T: Metadata + 'static,
	{
		fn type_id() -> Type {
			TypeCustom::new(
				"MyStruct",
				Namespace::from_module_path(module_path!()).unwrap(),
				tuple_meta_type!(T),
				TypeDefStruct::new(vec![NamedField::new("data", T::meta_type())]).into(),
			)
			.into()
		}
	}

	// Normal struct
	let struct_bool_id = TypeCustom::new(
		"MyStruct",
		Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
		tuple_meta_type!(bool),
		TypeDefStruct::new(vec![NamedField::new("data", bool::meta_type())]).into(),
	);
	assert_type_id!(MyStruct<bool>, struct_bool_id.clone());

	// With "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	let expected_type_id = TypeCustom::new(
		"MyStruct",
		Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
		vec![<Box<MyStruct<bool>>>::meta_type()],
		TypeDefStruct::new(vec![NamedField::new("data", <Box<MyStruct<bool>>>::meta_type())]).into(),
	);
	assert_type_id!(SelfTyped, expected_type_id);
}
