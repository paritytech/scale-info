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

use crate::*;
use core::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec};

fn assert_type<T, E>(expected_type: E, expected_path: Path, expected_params: Vec<MetaTypeParameter>)
where
	T: TypeInfo + ?Sized,
	E: Into<Type>,
{
	assert_eq!(T::type_info(), expected_type.into());
	assert_eq!(T::path(), expected_path);
	assert_eq!(T::params(), expected_params);
}

// todo: share all these macros with derive tests?
macro_rules! assert_type {
	( $ty:ty, $expected_ty:expr, $expected_path:expr, $expected_params:expr ) => {{
		assert_type::<$ty, _>($expected_ty, $expected_path, $expected_params)
		}};
}

macro_rules! assert_primitive {
	( $ty:ty, $expected_ty:expr) => {{
		assert_type::<$ty, _>($expected_ty, Path::voldemort(), Vec::new())
		}};
}

macro_rules! type_param {
	( $parent:ty, $ty:ty, $name:ident ) => {
		$crate::MetaTypeParameter::new::<$parent, $ty>(stringify!($name)).into()
	};
}

macro_rules! type_params {
	( $parent:ty, $(($ty:ty, $name:ident)),* ) => {
		{
			let mut v = Vec::new();
			$(
				v.push(type_param!($parent, $ty, $name));
			)*
			v
		}
	}
}

#[test]
fn primitives() {
	assert_primitive!(bool, TypePrimitive::Bool);
	assert_primitive!(&str, TypePrimitive::Str);
	assert_primitive!(String, TypePrimitive::Str);
	assert_primitive!(i8, TypePrimitive::I8);
}

#[test]
fn prelude_items() {
	assert_type!(
		[bool],
		TypeSequence::new(bool::meta_type()),
		Path::prelude("Sequence"),
		type_params!([bool], (bool, T))
	);

	assert_type!(
		Option<u128>,
		TypeVariant::new(
			Variants::with_fields()
				.variant_unit("None")
				.variant("Some", Fields::unnamed().parameter_field::<Option<u128>, u128>("T"))
		),
		Path::prelude("Option"),
		type_params!(Option<u128>, (u128, T))
	);
	assert_type!(
		Result<bool, String>,
		TypeVariant::new(
			Variants::with_fields()
				.variant("Ok", Fields::unnamed().parameter_field::<Result<bool, String>, bool>("T"))
				.variant("Err", Fields::unnamed().parameter_field::<Result<bool, String>, String>("E"))
		),
		Path::prelude("Result"),
		type_params!(Result<bool, String>, (bool, T), (String, E))
	);
	assert_type!(
		PhantomData<i32>,
		TypeComposite::unit(),
		Path::prelude("PhantomData"),
		type_params!(PhantomData<i32>, (i32, T))
	);
}

#[test]
fn tuple_primitives() {
	// unit
	assert_type!((), TypeTuple::new(vec![]), Path::prelude("Tuple"), vec![]);

	// tuple with one element
	let type_params = type_params!((bool,), (bool, A));
	assert_type!(
		(bool,),
		TypeTuple::new(type_params.clone()),
		Path::prelude("Tuple1"),
		type_params
	);

	// tuple with multiple elements
	let type_params = type_params!((bool, String), (bool, A), (String, B));
	assert_type!(
		(bool, String),
		TypeTuple::new(type_params.clone()),
		Path::prelude("Tuple2"),
		type_params
	);

	// nested tuple
	let type_params = type_params!(((i8, i16), (u32, u64)), ((i8, i16), A), ((u32, u64), B));
	assert_type!(
		((i8, i16), (u32, u64)),
		TypeTuple::new(type_params.clone()),
		Path::prelude("Tuple2"),
		type_params
	);
}

#[test]
fn array_primitives() {
	// array
	assert_type!(
		[bool; 3],
		TypeArray::new(3, bool::meta_type()),
		Path::voldemort(),
		type_params!([bool; 3], (bool, T))
	);
	// nested
	assert_type!(
		[[i32; 5]; 5],
		TypeArray::new(5, <[i32; 5]>::meta_type()),
		Path::voldemort(),
		type_params!([[i32; 5]; 5], ([i32; 5], T))
	);
	// sequence
	assert_type!(
		[bool],
		TypeSequence::new(bool::meta_type()),
		Path::prelude("Sequence"),
		type_params!([bool], (bool, T))
	);
	// vec
	assert_type!(
		Vec<bool>,
		TypeSequence::new(bool::meta_type()),
		Path::prelude("Sequence"),
		type_params!(Vec<bool>, (bool, T))
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
		fn path() -> Path {
			Path::new("MyStruct", module_path!())
		}

		fn params() -> Vec<MetaTypeParameter> {
			vec![MetaTypeParameter::new::<Self, T>("T")]
		}

		fn type_info() -> Type {
			TypeComposite::new(Fields::named().field_of::<T>("data")).into()
		}
	}

	// todo: [AJ] fix up this test
	// // Normal struct
	// let struct_bool_type_info = TypeComposite::new()
	// 	.path(Path::from_segments(vec!["scale_info", "tests", "MyStruct"]))
	// 	.type_params(tuple_meta_type!(bool))
	// 	.fields(Fields::named().field_of::<bool>("data"));
	//
	// assert_type!(MyStruct<bool>, struct_bool_type_info);
	//
	// // With "`Self` typed" fields
	// type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	// let expected_type = TypeComposite::new(
	// 	Fields::named().field_of::<Box<MyStruct<bool>>>("data")
	// )
	// 	.path(Path::new("MyStruct", "scale_info::tests"))
	// 	.type_params(tuple_meta_type!(Box<MyStruct<bool>>))
	// assert_type!(SelfTyped, expected_type);
}
