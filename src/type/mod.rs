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

mod product;
mod sum;
mod fields;
mod path;

pub use self::{
	path::*,
	fields::*,
	product::*,
	sum::*,
};

/// Implementors return their meta type identifiers.
pub trait HasType {
	/// Returns the static type identifier for `Self`.
	// todo: [AJ] good name for this? r#type() perhaps?
	fn get_type() -> Type;
}

/// A type identifier.
///
/// This uniquely identifies types and can be used to refer to type definitions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "
	F::Type: Serialize,
	F::IndirectType: Serialize
")]
#[serde(rename_all = "camelCase")]
pub enum Type<F: Form = MetaForm> {
	/// A product type (e.g. a struct)
	Product(TypeProduct<F>),
	/// A sum type (e.g. an enum)
	Sum(TypeSum<F>),
	/// A slice type with runtime known length.
	Slice(TypeSlice<F>),
	/// An array type with compile-time known length.
	Array(TypeArray<F>),
	/// A dynamic collection e.g. Vec<T>, BTreeMap<K, V>
	Collection(TypeCollection<F>),
	/// A tuple type.
	Tuple(TypeTuple<F>),
	/// A Rust primitive type.
	Primitive(TypePrimitive),
}

impl IntoCompact for Type {
	type Output = Type<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			Type::Product(product) => product.into_compact(registry).into(),
			Type::Sum(sum) => sum.into_compact(registry).into(),
			Type::Slice(slice) => slice.into_compact(registry).into(),
			Type::Array(array) => array.into_compact(registry).into(),
			Type::Collection(collection) => collection.into_compact(registry).into(),
			Type::Tuple(tuple) => tuple.into_compact(registry).into(),
			Type::Primitive(primitive) => primitive.into(),
		}
	}
}

impl From<TypeProductStruct> for Type {
	fn from(ty: TypeProductStruct) -> Type {
		ty.into()
	}
}

impl From<TypeProductTupleStruct> for Type {
	fn from(ty: TypeProductTupleStruct) -> Type {
		ty.into()
	}
}

impl From<TypeSumEnum> for Type {
	fn from(ty: TypeSumEnum) -> Type {
		ty.into()
	}
}

impl From<TypeSumClikeEnum> for Type {
	fn from(ty: TypeSumClikeEnum) -> Type {
		ty.into()
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
#[serde(bound = "F::IndirectType: Serialize")]
pub struct TypeArray<F: Form = MetaForm> {
	/// The length of the array type definition.
	pub len: u16,
	/// The element type of the array type definition.
	#[serde(rename = "type")]
	pub type_param: F::IndirectType,
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

/// A type identifier for collection type definitions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::IndirectType: Serialize")]
pub struct TypeCollection<F: Form = MetaForm> {
	/// The name of the collection type.
	name: F::String,
	/// The element type of the collection.
	#[serde(rename = "type")]
	element_type: F::IndirectType,
}

impl IntoCompact for TypeCollection {
	type Output = TypeCollection<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeCollection {
			name: registry.register_string(self.name),
			element_type: registry.register_type(&self.element_type),
		}
	}
}

impl TypeCollection {
	/// Creates a new type identifier to refer to a custom type definition.
	pub fn new(name: &'static str, type_param: MetaType) -> Self {
		Self {
			name,
			element_type: type_param,
		}
	}

	/// Creates a new type identifier to refer to collection type definitions.
	///
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn of<T>(name: &'static str) -> Self
	where
		T: Metadata + 'static,
	{
		Self::new(name, MetaType::new::<T>())
	}
}

/// A type identifier to refer to tuple types.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::Type: Serialize")]
#[serde(transparent)]
pub struct TypeTuple<F: Form = MetaForm> {
	/// The types in the tuple type definition.
	pub type_params: Vec<F::Type>,
}

impl IntoCompact for TypeTuple {
	type Output = TypeTuple<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeTuple {
			type_params: self
				.type_params
				.into_iter()
				.map(|param| registry.register_type(&param))
				.collect::<Vec<_>>(),
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
#[serde(bound = "F::IndirectType: Serialize")]
pub struct TypeSlice<F: Form = MetaForm> {
	/// The element type of the slice type definition.
	#[serde(rename = "type")]
	type_param: F::IndirectType,
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
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn of<T>() -> Self
	where
		T: Metadata + 'static,
	{
		Self::new(MetaType::new::<T>())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn namespace_ok() {
		assert_eq!(
			Namespace::new(vec!["hello"]),
			Ok(Namespace {
				segments: vec!["hello"]
			})
		);
		assert_eq!(
			Namespace::new(vec!["Hello", "World"]),
			Ok(Namespace {
				segments: vec!["Hello", "World"]
			})
		);
		assert_eq!(Namespace::new(vec!["_"]), Ok(Namespace { segments: vec!["_"] }));
	}

	#[test]
	fn namespace_err() {
		assert_eq!(Namespace::new(vec![]), Err(NamespaceError::MissingSegments));
		assert_eq!(
			Namespace::new(vec![""]),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
		assert_eq!(
			Namespace::new(vec!["1"]),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
		assert_eq!(
			Namespace::new(vec!["Hello", ", World!"]),
			Err(NamespaceError::InvalidIdentifier { segment: 1 })
		);
	}

	#[test]
	fn namespace_from_module_path() {
		assert_eq!(
			Namespace::from_module_path("hello::world"),
			Ok(Namespace {
				segments: vec!["hello", "world"]
			})
		);
		assert_eq!(
			Namespace::from_module_path("::world"),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
	}
}
