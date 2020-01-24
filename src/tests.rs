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

fn assert_type<T, E>(expected: E)
where
	T: HasType + ?Sized,
	E: Into<Type>,
{
	assert_eq!(T::get_type(), expected.into());
}

macro_rules! assert_type {
	( $ty:ty, $expected:expr ) => {{
		assert_type::<$ty, _>($expected)
		}};
}

#[test]
fn primitives() {
	assert_type!(bool, TypePrimitive::Bool);
	assert_type!(&str, TypePrimitive::Str);
	assert_type!(i8, TypePrimitive::I8);

	assert_type!([bool], TypeSlice::new(bool::meta_type()));
}

#[test]
fn prelude_items() {
	assert_type!(
		String,
		TypeProductStruct::new(
			TypePath::new("String", Namespace::prelude(), Vec::new()),
			vec![NamedField::new("vec", MetaType::new::<Vec<u8>>())]
		)
	);

	assert_type!(
		Option<u128>,
		TypeSumEnum::new(
			TypePath::new("Option", Namespace::prelude(), tuple_meta_type!(u128)),
			vec![
				EnumVariantUnit::new("None").into(),
				EnumVariantTupleStruct::new("Some", vec![UnnamedField::of::<u128>()]).into(),
			],
		)
	);
	assert_type!(
		Result<bool, String>,
		TypeSumEnum::new(
			TypePath::new("Result", Namespace::prelude(), tuple_meta_type!(bool, String)),
			vec![
				EnumVariantTupleStruct::new("Ok", vec![UnnamedField::of::<bool>()]).into(),
				EnumVariantTupleStruct::new("Err", vec![UnnamedField::of::<String>()]).into(),
			]
		)
	);
	assert_type!(
		PhantomData<i32>,
		TypeProductTupleStruct::new(
			TypePath::new("PhantomData", Namespace::prelude(), tuple_meta_type!(i32)),
			vec![],
		)
	)
}

#[test]
fn tuple_primitives() {
	// unit
	assert_type!((), TypeTuple::new(tuple_meta_type!()));

	// tuple with one element
	assert_type!((bool,), TypeTuple::new(tuple_meta_type!(bool)));

	// tuple with multiple elements
	assert_type!((bool, String), TypeTuple::new(tuple_meta_type!(bool, String)));

	// nested tuple
	assert_type!(
		((i8, i16), (u32, u64)),
		TypeTuple::new(vec![<(i8, i16)>::meta_type(), <(u32, u64)>::meta_type(),])
	);
}

#[test]
fn array_primitives() {
	// array
	assert_type!([bool; 3], TypeArray::new(3, bool::meta_type()));
	// nested
	assert_type!([[i32; 5]; 5], TypeArray::new(5, <[i32; 5]>::meta_type()));
	// slice
	assert_type!([bool], TypeSlice::new(bool::meta_type()));
	// vec
	assert_type!(Vec<bool>, TypeCollection::of::<bool>("Vec"));
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
		fn get_type() -> Type {
			TypeProductStruct::new(
				TypePath::new(
					"MyStruct",
					Namespace::from_module_path(module_path!()).unwrap(),
					tuple_meta_type!(T),
				),
				vec![NamedField::new("data", T::meta_type())],
			).into()
		}
	}

	// Normal struct
	let struct_bool_id = TypeProductStruct::new(
		TypePath::new("MyStruct", Namespace::new(vec!["type_metadata", "tests"]).unwrap(), tuple_meta_type!(bool)),
		vec![NamedField::new("data", bool::meta_type())]
	).into();
	assert_type!(MyStruct<bool>, struct_bool_id.clone());

	// With "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	let expected_type_id = TypeProductStruct::new(
		TypePath::new(
			"MyStruct",
			Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
			vec![<Box<MyStruct<bool>>>::meta_type()],
		),
		vec![NamedField::new("data", <Box<MyStruct<bool>>>::meta_type())]
	).into();
	assert_type!(SelfTyped, expected_type_id);
}
