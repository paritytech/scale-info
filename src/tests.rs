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
fn primitives() {
	assert_type_id!(bool, TypeIdPrimitive::Bool);
	assert_type_id!(String, TypeIdPrimitive::Str);
	assert_type_id!(&str, TypeIdPrimitive::Str);
	assert_type_id!(i8, TypeIdPrimitive::I8);

	assert_type_id!(Box<String>, TypeIdPrimitive::Str);
	assert_type_id!(&String, TypeIdPrimitive::Str);
	assert_type_id!([bool], TypeIdSlice::new(bool::meta_type()));
}

#[test]
fn prelude_items() {
	assert_type_id!(
		Option<u128>,
		TypeIdCustom::new("Option", Namespace::prelude(), tuple_meta_type!(u128))
	);
	assert_type_id!(
		Result<bool, String>,
		TypeIdCustom::new("Result", Namespace::prelude(), tuple_meta_type!(bool, String))
	);
	assert_type_id!(
		PhantomData<i32>,
		TypeIdCustom::new("PhantomData", Namespace::prelude(), tuple_meta_type!(i32))
	)
}

#[test]
fn tuple_primitives() {
	// unit
	assert_type_id!((), TypeIdTuple::new(tuple_meta_type!()));

	// tuple with one element
	assert_type_id!((bool,), TypeIdTuple::new(tuple_meta_type!(bool)));

	// tuple with multiple elements
	assert_type_id!((bool, String), TypeIdTuple::new(tuple_meta_type!(bool, String)));

	// nested tuple
	assert_type_id!(
		((i8, i16), (u32, u64)),
		TypeIdTuple::new(vec![<(i8, i16)>::meta_type(), <(u32, u64)>::meta_type(),])
	);
}

#[test]
fn array_primitives() {
	// array
	assert_type_id!([bool; 3], TypeIdArray::new(3, bool::meta_type()));
	// nested
	assert_type_id!([[i32; 5]; 5], TypeIdArray::new(5, <[i32; 5]>::meta_type()));
	// slice
	assert_type_id!([bool], TypeIdSlice::new(bool::meta_type()));
	// vec
	assert_type_id!(
		Vec<bool>,
		TypeIdCustom::new("Vec", Namespace::prelude(), tuple_meta_type![bool])
	);
}

#[test]
fn struct_with_generics() {
	#[allow(unused)]
	struct MyStruct<T> {
		data: T,
	}

	impl<T> HasTypeId for MyStruct<T>
	where
		T: Metadata + 'static,
	{
		fn type_id() -> TypeId {
			TypeIdCustom::new(
				"MyStruct",
				Namespace::from_module_path(module_path!()).unwrap(),
				tuple_meta_type!(T),
			)
			.into()
		}
	}

	impl<T> HasTypeDef for MyStruct<T>
	where
		T: Metadata,
	{
		fn type_def() -> TypeDef {
			TypeDefStruct::new(vec![NamedField::new("data", T::meta_type())]).into()
		}
	}

	// Normal struct
	let struct_bool_id = TypeIdCustom::new(
		"MyStruct",
		Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
		tuple_meta_type!(bool),
	);
	assert_type_id!(MyStruct<bool>, struct_bool_id.clone());

	let struct_bool_def = TypeDefStruct::new(vec![NamedField::new("data", bool::meta_type())]).into();
	assert_eq!(<MyStruct<bool>>::type_def(), struct_bool_def);

	// With "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	let expected_type_id = TypeIdCustom::new(
		"MyStruct",
		Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
		vec![<Box<MyStruct<bool>>>::meta_type()],
	);
	assert_type_id!(SelfTyped, expected_type_id);
	assert_eq!(
		SelfTyped::type_def(),
		TypeDefStruct::new(vec![NamedField::new("data", <Box<MyStruct<bool>>>::meta_type()),]).into(),
	);
}
