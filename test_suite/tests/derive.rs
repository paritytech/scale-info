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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec, vec::Vec};

use pretty_assertions::assert_eq;
use scale_info::{
	Fields, MetaTypeParameter, MetaTypeParameterValue, Metadata, Path, Type, TypeComposite, TypeInfo,
	TypeVariant, Variants,
};

fn assert_type<T, E>(expected_type: E, expected_path: &Path, expected_params: Vec<MetaTypeParameter>)
where
	T: TypeInfo + ?Sized,
	E: Into<Type>,
{
	assert_eq!(T::type_info(), expected_type.into());
	assert_eq!(T::path(), *expected_path);
	assert_eq!(T::params(), expected_params);
}

macro_rules! assert_type {
	( $ty:ty, $expected_ty:expr, $expected_path:expr, $expected_params:expr ) => {{
		assert_type::<$ty, _>($expected_ty, $expected_path, $expected_params)
		}};
}

macro_rules! type_param {
	( $parent:ty, $ty:ty, $name:ident ) => {
		$crate::MetaTypeParameter::new::<$parent, $ty>(stringify!($name))
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
fn struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S<T, U> {
		pub t: T,
		pub u: U,
	}

	type ConcreteS = S<bool, u8>;

	let path = Path::new("S", "derive");
	let params = type_params!(ConcreteS, (bool, T), (u8, U));
	let struct_type = TypeComposite::new(
		Fields::named()
			.parameter_field::<ConcreteS, bool>("t", "T")
			.parameter_field::<ConcreteS, u8>("u", "U"),
	);

	assert_type!(ConcreteS, struct_type, &path, params);

	// With "`Self` typed" fields

	type SelfTyped = S<Box<S<bool, u8>>, bool>;

	let params = type_params!(SelfTyped, (Box<S<bool, u8>>, T), (bool, U));
	let self_typed_type = TypeComposite::new(
		Fields::named()
			.parameter_field::<SelfTyped, Box<S<bool, u8>>>("t", "T")
			.parameter_field::<SelfTyped, bool>("u", "U"),
	);
	assert_type!(SelfTyped, self_typed_type, &path, params);
}

#[test]
fn parameterized_concrete_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct ConcreteParameterized {
		a: Option<bool>,
		b: Option<u32>,
	}

	let path = Path::new("ConcreteParameterized", "derive");
	let struct_type = TypeComposite::new(
		Fields::named()
			.parameterized_field::<Option<bool>>("a", vec![MetaTypeParameterValue::concrete::<bool>()])
			.parameterized_field::<Option<u32>>("b", vec![MetaTypeParameterValue::concrete::<u32>()]),
	);

	assert_type!(ConcreteParameterized, struct_type, &path, Vec::new())
}

#[test]
fn parameterized_generic_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct GenericParameterized<T> {
		a: Option<T>,
	}

	let path = Path::new("GenericParameterized", "derive");
	let params = type_params!(GenericParameterized<u8>, (u8, T));
	let struct_type = TypeComposite::new(Fields::named().parameterized_field::<Option<u8>>(
		"a",
		vec![MetaTypeParameterValue::parameter::<GenericParameterized<u8>, u8>("T")],
	));

	assert_type!(GenericParameterized<u8>, struct_type, &path, params)
}

#[test]
fn parameterized_tuple_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct TupleParameterized<T, U, V> {
		a: (T, U),
		b: (T, U, V),
	}

	let path = Path::new("TupleParameterized", "derive");
	let params = type_params!(TupleParameterized<u8, u16, u32>, (u8, T), (u16, U), (u32, V));
	let struct_type = TypeComposite::new(
		Fields::named()
			.parameterized_field::<(u8, u16)>(
				"a",
				vec![
					MetaTypeParameterValue::parameter::<TupleParameterized<u8, u16, u32>, u8>("T"),
					MetaTypeParameterValue::parameter::<TupleParameterized<u8, u16, u32>, u16>("U"),
				],
			)
			.parameterized_field::<(u8, u16, u32)>(
				"b",
				vec![
					MetaTypeParameterValue::parameter::<TupleParameterized<u8, u16, u32>, u8>("T"),
					MetaTypeParameterValue::parameter::<TupleParameterized<u8, u16, u32>, u16>("U"),
					MetaTypeParameterValue::parameter::<TupleParameterized<u8, u16, u32>, u32>("V"),
				],
			),
	);

	assert_type!(TupleParameterized<u8, u16, u32>, struct_type, &path, params)
}

#[test]
fn parameterized_array_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct ArrayParameterized<T, U> {
		a: [T; 8],
		b: [(T, U); 16],
	}

	let path = Path::new("ArrayParameterized", "derive");
	let params = type_params!(ArrayParameterized<u8, u16>, (u8, T), (u16, U));
	let struct_type = TypeComposite::new(
		Fields::named()
			.parameterized_field::<[u8; 8]>(
				"a",
				vec![MetaTypeParameterValue::parameter::<ArrayParameterized<u8, u16>, u8>(
					"T",
				)],
			)
			.parameterized_field::<[(u8, u16); 16]>(
				"b",
				vec![
					MetaTypeParameterValue::parameter::<ArrayParameterized<u8, u16>, u8>("T"),
					MetaTypeParameterValue::parameter::<ArrayParameterized<u8, u16>, u16>("U"),
				],
			),
	);

	assert_type!(ArrayParameterized<u8, u16>, struct_type, &path, params)
}

#[test]
fn parameterized_refs_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct RefsParameterized<'a, T, U> {
		a: &'a T,
		b: &'a mut U,
		c: (&'a mut T, &'a U),
	}

	let path = Path::new("RefsParameterized", "derive");
	let params = type_params!(RefsParameterized<'static, u8, u16>, (u8, T), (u16, U));
	let struct_type = TypeComposite::new(
		Fields::named()
			// refs are stripped to the owned type e.g. &T and &mut T become T, since SCALE encodes
			// all forms to the same representation.
			.parameter_field::<RefsParameterized<'static, u8, u16>, u8>("a", "T")
			.parameter_field::<RefsParameterized<'static, u8, u16>, u16>("b", "U")
			.parameterized_field::<(&'static mut u8, &'static u16)>(
				"c",
				vec![
					MetaTypeParameterValue::parameter::<RefsParameterized<'static, u8, u16>, u8>("T"),
					MetaTypeParameterValue::parameter::<RefsParameterized<'static, u8, u16>, u16>("U"),
				],
			),
	);

	assert_type!(RefsParameterized<'static, u8, u16>, struct_type, &path, params)
}

#[test]
fn tuple_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S<T>(T);

	type ConcreteS = S<bool>;

	let path = Path::new("S", "derive");
	let params = type_params!(ConcreteS, (bool, T));
	let ty = TypeComposite::new(Fields::unnamed().parameter_field::<ConcreteS, bool>("T"));

	assert_type!(ConcreteS, ty, &path, params);
}

#[test]
fn unit_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S;

	let path = Path::new("S", "derive");
	let params = Vec::new();
	let ty = TypeComposite::unit();

	assert_type!(S, ty, &path, params);
}

#[test]
fn c_like_enum_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	enum E {
		A,
		B = 10,
	}

	let path = Path::new("E", "derive");
	let params = Vec::new();
	let ty = TypeVariant::new(Variants::with_discriminants().variant("A", 0u64).variant("B", 10u64));

	assert_type!(E, ty, &path, params);
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

	let path = Path::new("E", "derive");
	let params = type_params!(E<bool>, (bool, T));
	let ty = TypeVariant::new(
		Variants::with_fields()
			.variant("A", Fields::unnamed().parameter_field::<E<bool>, bool>("T"))
			.variant("B", Fields::named().parameter_field::<E<bool>, bool>("b", "T"))
			.variant_unit("C"),
	);

	assert_type!(E<bool>, ty, &path, params);
}
