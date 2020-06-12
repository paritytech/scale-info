// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use crate::build::*;
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
	assert_type!(bool, TypeDefPrimitive::Bool);
	assert_type!(&str, TypeDefPrimitive::Str);
	assert_type!(i8, TypeDefPrimitive::I8);

	assert_type!([bool], TypeDefSequence::new(bool::meta_type()));
}

#[test]
fn prelude_items() {
	assert_type!(String, TypeDefPrimitive::Str);

	assert_type!(
		Option<u128>,
		Type::builder()
			.path(Path::prelude("Option"))
			.type_params(tuple_meta_type!(u128))
			.variant(
				Variants::with_fields()
					.variant_unit("None")
					.variant("Some", Fields::unnamed().field_of::<u128>())
			)
	);
	assert_type!(
		Result<bool, String>,
		Type::builder()
			.path(Path::prelude("Result"))
			.type_params(tuple_meta_type!(bool, String))
			.variant(
				Variants::with_fields()
					.variant("Ok", Fields::unnamed().field_of::<bool>())
					.variant("Err", Fields::unnamed().field_of::<String>())
			)
	);
	assert_type!(
		PhantomData<i32>,
		Type::builder()
			.path(Path::prelude("PhantomData"))
			.type_params(tuple_meta_type!(i32))
			.composite(Fields::unit())
	);
}

#[test]
fn tuple_primitives() {
	// unit
	assert_type!((), TypeDefTuple::new(tuple_meta_type!()));

	// tuple with one element
	assert_type!((bool,), TypeDefTuple::new(tuple_meta_type!(bool)));

	// tuple with multiple elements
	assert_type!((bool, String), TypeDefTuple::new(tuple_meta_type!(bool, String)));

	// nested tuple
	assert_type!(
		((i8, i16), (u32, u64)),
		TypeDefTuple::new(vec![<(i8, i16)>::meta_type(), <(u32, u64)>::meta_type(),])
	);
}

#[test]
fn array_primitives() {
	// array
	assert_type!([bool; 3], TypeDefArray::new(3, bool::meta_type()));
	// nested
	assert_type!([[i32; 5]; 5], TypeDefArray::new(5, <[i32; 5]>::meta_type()));
	// sequence
	assert_type!([bool], TypeDefSequence::new(bool::meta_type()));
	// vec
	assert_type!(Vec<bool>, TypeDefSequence::new(bool::meta_type()));
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
			Type::builder()
				.path(Path::new("MyStruct", module_path!()))
				.type_params(tuple_meta_type!(T))
				.composite(Fields::named().field_of::<T>("data"))
				.into()
		}
	}

	// Normal struct
	let struct_bool_type_info = Type::builder()
		.path(Path::from_segments(vec!["scale_info", "tests", "MyStruct"]).unwrap())
		.type_params(tuple_meta_type!(bool))
		.composite(Fields::named().field_of::<bool>("data"));

	assert_type!(MyStruct<bool>, struct_bool_type_info);

	// With "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	let expected_type = Type::builder()
		.path(Path::new("MyStruct", "scale_info::tests"))
		.type_params(tuple_meta_type!(Box<MyStruct<bool>>))
		.composite(Fields::named().field_of::<Box<MyStruct<bool>>>("data"));
	assert_type!(SelfTyped, expected_type);
}
