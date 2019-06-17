// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of type-metadata.
//
// type-metadata is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// type-metadata is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with type-metadata.  If not, see <http://www.gnu.org/licenses/>.

use crate::*;
use std::marker::PhantomData;

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
	assert_type_id!([bool], TypeIdSlice::new(TypeIdPrimitive::Bool));
	assert_type_id!(PhantomData<bool>, TypeIdPrimitive::Bool);
}

#[test]
fn prelude_items() {
	assert_type_id!(
		Option<u128>,
		TypeIdCustom::new("Option", Namespace::prelude(), tuple_type_id!(u128))
	);
	assert_type_id!(
		Result<bool, String>,
		TypeIdCustom::new("Result", Namespace::prelude(), tuple_type_id!(bool, str))
	);
}

#[test]
fn tuple_primitives() {
	// unit
	assert_type_id!((), TypeIdTuple::new(tuple_type_id!()));

	// tuple with one element
	assert_type_id!((bool,), TypeIdTuple::new(tuple_type_id!(bool)));

	// tuple with multiple elements
	assert_type_id!((bool, String), TypeIdTuple::new(tuple_type_id!(bool, String)));

	// nested tuple
	assert_type_id!(
		((i8, i16), (u32, u64)),
		TypeIdTuple::new(vec![
			TypeIdTuple::new(tuple_type_id!(i8, i16)).into(),
			TypeIdTuple::new(tuple_type_id!(u32, u64)).into(),
		])
	);
}

#[test]
fn array_primitives() {
	// array
	assert_type_id!([bool; 3], TypeIdArray::new(3, bool::type_id()));
	// nested
	assert_type_id!([[i32; 5]; 5], TypeIdArray::new(5, TypeIdArray::new(5, i32::type_id())));
	// slice
	assert_type_id!([bool], TypeIdSlice::new(bool::type_id()));
	// vec
	assert_type_id!(
		Vec<bool>,
		TypeIdCustom::new("Vec", Namespace::prelude(), tuple_type_id![bool])
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
		T: HasTypeId,
	{
		fn type_id() -> TypeId {
			TypeIdCustom::new(
				"MyStruct",
				Namespace::from_str(module_path!()).unwrap(),
				tuple_type_id!(T),
			)
			.into()
		}
	}

	impl<T> HasTypeDef for MyStruct<T>
	where
		T: Metadata,
	{
		fn type_def() -> TypeDef {
			TypeDefStruct::new(vec![NamedField::new("data", T::type_id())]).into()
		}
	}

	impl<T> RegisterSubtypes for MyStruct<T>
	where
		T: Metadata,
	{
		fn register_subtypes(registry: &mut Registry) {
			registry.register_type::<T>();
		}
	}

	// Normal struct
	let struct_bool_id = TypeIdCustom::new(
		"MyStruct",
		Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
		tuple_type_id!(bool),
	);
	assert_type_id!(MyStruct<bool>, struct_bool_id.clone());

	let struct_bool_def = TypeDefStruct::new(vec![NamedField::new("data", bool::type_id())]).into();
	assert_eq!(<MyStruct<bool>>::type_def(), struct_bool_def);

	// With "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	let expected_type_id = TypeIdCustom::new(
		"MyStruct",
		Namespace::new(vec!["type_metadata", "tests"]).unwrap(),
		vec![struct_bool_id.clone().into()],
	);
	assert_type_id!(SelfTyped, expected_type_id);
	assert_eq!(
		SelfTyped::type_def(),
		TypeDefStruct::new(vec![NamedField::new("data", struct_bool_id.clone()),]).into(),
	);
}
