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
use alloc::{
	boxed::Box,
	vec::Vec
};

use scale_info::{tuple_meta_type, Fields, Metadata, Path, Type, TypeComposite, TypeInfo, TypeVariant, Variants, MetaTypeParameter};

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
	( $parent:ty, $ty:ty ) => {
		$crate::MetaTypeParameter::new::<$parent, $ty>(stringify!($ty)).into()
	};
}

macro_rules! type_params {
	( $parent:ty, $($ty:ty),* ) => {
		{
			let mut v = Vec::new();
			$(
				v.push(type_param!($parent, $ty));
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
	let params = type_params!(ConcreteS, bool, u8);
	let struct_type = TypeComposite::new(
		Fields::named()
			.parameter_field::<ConcreteS, bool>("t", "T")
			.parameter_field::<ConcreteS, u8>("u", "U")
	);

	assert_type!(ConcreteS, struct_type, &path, params);

	// With "`Self` typed" fields

	type SelfTyped = S<Box<S<bool, u8>>, bool>;

	let params = type_params!(SelfTyped, Box<S<bool, u8>>, bool);
	let self_typed_type = TypeComposite::new(
		Fields::named()
			.field_of::<Box<S<bool, u8>>>("t")
			.field_of::<bool>("u")
	);
	assert_type!(SelfTyped, self_typed_type, &path, params);
}

#[test]
fn tuple_struct_derive() {
	#[allow(unused)]
	#[derive(Metadata)]
	struct S<T>(T);

	type ConcreteS = S<bool>;

	let path = Path::new("S", "derive");
	let params = type_params!(ConcreteS, bool);
	let ty = TypeComposite::new(
		Fields::unnamed()
			.parameter_field::<ConcreteS, bool>("T")
	);

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
	let ty = TypeVariant::new(
		Variants::with_discriminants()
			.variant("A", 0u64)
			.variant("B", 10u64)
	);

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
	let params = type_params!(E<bool>, bool);
	let ty = TypeVariant::new(
		Variants::with_fields()
			.variant("A", Fields::unnamed().field_of::<bool>())
			.variant("B", Fields::named().field_of::<bool>("b"))
			.variant_unit("C"),
	);

	assert_type!(E<bool>, ty, &path, params);
}
