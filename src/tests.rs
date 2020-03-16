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
		TypeComposite::new("String", Namespace::prelude()).fields(Fields::named().field_of::<Vec<u8>>("vec"))
	);

	assert_type!(
		Option<u128>,
		TypeVariant::new("Option", Namespace::prelude())
			.type_params(tuple_meta_type!(u128))
			.variants(
				Variants::with_fields()
					.variant_unit("None")
					.variant("Some", Fields::unnamed().field_of::<u128>())
			)
	);
	assert_type!(
		Result<bool, String>,
		TypeVariant::new("Result", Namespace::prelude())
			.type_params(tuple_meta_type!(bool, String))
			.variants(
				Variants::with_fields()
					.variant("Ok", Fields::unnamed().field_of::<bool>())
					.variant("Err", Fields::unnamed().field_of::<String>())
			)
	);
	assert_type!(
		PhantomData<i32>,
		TypeComposite::new("PhantomData", Namespace::prelude())
			.type_params(tuple_meta_type!(i32))
			.unit()
	);
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
	assert_type!(
		Vec<bool>,
		TypeComposite::new("Vec", Namespace::prelude())
			.type_params(tuple_meta_type!(bool))
			.fields(Fields::named().field_of::<[bool]>("elems"))
	);
}

#[test]
fn struct_with_generics() {
	#[allow(unused)]
	struct MyStruct<T> {
		data: T,
	}

	impl<T> TypeInfo for MyStruct<T>
	where
		T: Metadata + 'static,
	{
		fn type_info() -> Type {
			TypeComposite::new("MyStruct", Namespace::from_module_path(module_path!()).unwrap())
				.type_params(tuple_meta_type!(T))
				.fields(Fields::named().field_of::<T>("data"))
				.into()
		}
	}

	// Normal struct
	let struct_bool_type_info = TypeComposite::new("MyStruct", Namespace::new(vec!["type_metadata", "tests"]).unwrap())
		.type_params(tuple_meta_type!(bool))
		.fields(Fields::named().field_of::<bool>("data"));

	assert_type!(MyStruct<bool>, struct_bool_type_info);

	// With "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	let expected_type = TypeComposite::new("MyStruct", Namespace::new(vec!["type_metadata", "tests"]).unwrap())
		.type_params(tuple_meta_type!(Box<MyStruct<bool>>))
		.fields(Fields::named().field_of::<Box<MyStruct<bool>>>("data"));
	assert_type!(SelfTyped, expected_type);
}
