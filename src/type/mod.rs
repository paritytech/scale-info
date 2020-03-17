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

use crate::tm_std::*;

use crate::{
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Metadata, Registry,
};
use derive_more::From;
use serde::Serialize;

mod composite;
mod fields;
mod path;
mod variant;

pub use self::{composite::*, fields::*, path::*, variant::*};

/// A type identifier.
///
/// This uniquely identifies types and can be used to refer to type definitions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "camelCase")]
pub enum Type<F: Form = MetaForm> {
	/// A composite type (e.g. a struct or a tuple)
	Composite(TypeComposite<F>),
	/// A variant type (e.g. an enum)
	Variant(TypeVariant<F>),
	/// A slice type with runtime known length.
	Slice(TypeSlice<F>),
	/// An array type with compile-time known length.
	Array(TypeArray<F>),
	/// A tuple type.
	Tuple(TypeTuple<F>),
	/// A Rust primitive type.
	Primitive(TypePrimitive),
}

impl IntoCompact for Type {
	type Output = Type<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			Type::Composite(composite) => composite.into_compact(registry).into(),
			Type::Variant(variant) => variant.into_compact(registry).into(),
			Type::Slice(slice) => slice.into_compact(registry).into(),
			Type::Array(array) => array.into_compact(registry).into(),
			Type::Tuple(tuple) => tuple.into_compact(registry).into(),
			Type::Primitive(primitive) => primitive.into(),
		}
	}
}

/// Identifies a primitive Rust type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TypePrimitive {
	/// `bool` type
	Bool,
	/// `char` type
	Char,
	/// `str` type
	Str,
	/// `u8`
	U8,
	/// `u16`
	U16,
	/// `u32`
	U32,
	/// `u64`
	U64,
	/// `u128`
	U128,
	/// `i8`
	I8,
	/// `i16`
	I16,
	/// `i32`
	I32,
	/// `i64`
	I64,
	/// `i128`
	I128,
}

/// An array type identifier.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeArray<F: Form = MetaForm> {
	/// The length of the array type definition.
	pub len: u16,
	/// The element type of the array type definition.
	#[serde(rename = "type")]
	pub type_param: F::TypeId,
}

impl IntoCompact for TypeArray {
	type Output = TypeArray<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeArray {
			len: self.len,
			type_param: registry.register_type(&self.type_param),
		}
	}
}

impl TypeArray {
	/// Creates a new identifier to refer to array type definition.
	pub fn new(len: u16, type_param: MetaType) -> Self {
		Self { len, type_param }
	}
}

/// A type identifier to refer to tuple types.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(transparent)]
pub struct TypeTuple<F: Form = MetaForm> {
	/// The types in the tuple type definition.
	pub type_params: Vec<F::TypeId>,
}

impl IntoCompact for TypeTuple {
	type Output = TypeTuple<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeTuple {
			type_params: registry.register_types(self.type_params),
		}
	}
}

impl TypeTuple {
	/// Creates a new tuple type definition from the given types.
	pub fn new<T>(type_params: T) -> Self
	where
		T: IntoIterator<Item = MetaType>,
	{
		Self {
			type_params: type_params.into_iter().collect(),
		}
	}

	/// Creates a new unit tuple to represent the unit type, `()`.
	pub fn unit() -> Self {
		Self::new(vec![])
	}
}

/// A type identifier to refer to slice type definitions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeSlice<F: Form = MetaForm> {
	/// The element type of the slice type definition.
	#[serde(rename = "type")]
	type_param: F::TypeId,
}

impl IntoCompact for TypeSlice {
	type Output = TypeSlice<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeSlice {
			type_param: registry.register_type(&self.type_param),
		}
	}
}

impl TypeSlice {
	/// Creates a new type identifier to refer to slice type definitions.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(type_param: MetaType) -> Self {
		Self { type_param }
	}

	/// Creates a new type identifier to refer to slice type definitions.
	///
	/// Use this constructor if you want to instantiate from a given
	/// compile-time type.
	pub fn of<T>() -> Self
	where
		T: Metadata + 'static,
	{
		Self::new(MetaType::new::<T>())
	}
}
