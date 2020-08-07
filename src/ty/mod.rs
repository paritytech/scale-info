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

use crate::tm_std::*;

use crate::{
	build::TypeBuilder,
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Registry, TypeInfo,
};
use derive_more::From;
use scale::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod composite;
mod fields;
mod path;
mod variant;

pub use self::{composite::*, fields::*, path::*, variant::*};

/// A [`Type`] definition with optional metadata.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize, Deserialize, Encode, Decode)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct Type<F: Form = MetaForm> {
	/// The unique path to the type. Can be empty for built-in types
	#[serde(skip_serializing_if = "Path::is_empty")]
	path: Path<F>,
	/// The generic type parameters of the type in use. Empty for non generic types
	#[serde(rename = "params", skip_serializing_if = "Vec::is_empty")]
	type_params: Vec<F::TypeId>,
	/// The actual type definition
	#[serde(rename = "def")]
	type_def: TypeDef<F>,
}

impl IntoCompact for Type {
	type Output = Type<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		Type {
			path: self.path.into_compact(registry),
			type_params: registry.register_types(self.type_params),
			type_def: self.type_def.into_compact(registry),
		}
	}
}

impl From<TypeDefPrimitive> for Type {
	fn from(primitive: TypeDefPrimitive) -> Self {
		Self::new(Path::voldemort(), Vec::new(), primitive)
	}
}

impl From<TypeDefArray> for Type {
	fn from(array: TypeDefArray) -> Self {
		Self::new(Path::voldemort(), Vec::new(), array)
	}
}

impl From<TypeDefSequence> for Type {
	fn from(sequence: TypeDefSequence) -> Self {
		Self::new(Path::voldemort(), Vec::new(), sequence)
	}
}

impl From<TypeDefTuple> for Type {
	fn from(tuple: TypeDefTuple) -> Self {
		Self::new(Path::voldemort(), Vec::new(), tuple)
	}
}

impl Type {
	/// Create a [`TypeBuilder`](`crate::build::TypeBuilder`) the public API for constructing a [`Type`]
	pub fn builder() -> TypeBuilder {
		TypeBuilder::default()
	}

	pub(crate) fn new<I, D>(path: Path, type_params: I, type_def: D) -> Self
	where
		I: IntoIterator<Item = MetaType>,
		D: Into<TypeDef>,
	{
		Self {
			path,
			type_params: type_params.into_iter().collect(),
			type_def: type_def.into(),
			// marker: MetaForm,
		}
	}
}

/// The possible types a SCALE encodable Rust value could have.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize, Deserialize, Encode, Decode)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "camelCase")]
pub enum TypeDef<F: Form = MetaForm> {
	/// A composite type (e.g. a struct or a tuple)
	Composite(TypeDefComposite<F>),
	/// A variant type (e.g. an enum)
	Variant(TypeDefVariant<F>),
	/// A sequence type with runtime known length.
	Sequence(TypeDefSequence<F>),
	/// An array type with compile-time known length.
	Array(TypeDefArray<F>),
	/// A tuple type.
	Tuple(TypeDefTuple<F>),
	/// A Rust primitive type.
	Primitive(TypeDefPrimitive),
}

impl IntoCompact for TypeDef {
	type Output = TypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeDef::Composite(composite) => composite.into_compact(registry).into(),
			TypeDef::Variant(variant) => variant.into_compact(registry).into(),
			TypeDef::Sequence(sequence) => sequence.into_compact(registry).into(),
			TypeDef::Array(array) => array.into_compact(registry).into(),
			TypeDef::Tuple(tuple) => tuple.into_compact(registry).into(),
			TypeDef::Primitive(primitive) => primitive.into(),
		}
	}
}

/// A primitive Rust type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Encode, Decode, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TypeDefPrimitive {
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

/// An array type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Encode, Decode, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefArray<F: Form = MetaForm> {
	/// The length of the array type.
	pub len: u32,
	/// The element type of the array type.
	#[serde(rename = "type")]
	pub type_param: F::TypeId,
}

impl IntoCompact for TypeDefArray {
	type Output = TypeDefArray<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefArray {
			len: self.len,
			type_param: registry.register_type(&self.type_param),
		}
	}
}

impl TypeDefArray {
	/// Creates a new array type.
	pub fn new(len: u32, type_param: MetaType) -> Self {
		Self { len, type_param }
	}
}

/// A type to refer to tuple types.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Encode, Decode, Debug)]
#[serde(bound = "F::TypeId: Serialize + Deserialize")]
#[serde(transparent)]
pub struct TypeDefTuple<F: Form = MetaForm> {
	/// The types of the tuple fields.
	fields: Vec<F::TypeId>,
}

impl IntoCompact for TypeDefTuple {
	type Output = TypeDefTuple<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefTuple {
			fields: registry.register_types(self.fields),
		}
	}
}

impl TypeDefTuple {
	/// Creates a new tuple type definition from the given types.
	pub fn new<T>(type_params: T) -> Self
	where
		T: IntoIterator<Item = MetaType>,
	{
		Self {
			fields: type_params.into_iter().collect(),
		}
	}

	/// Creates a new unit tuple to represent the unit type, `()`.
	pub fn unit() -> Self {
		Self::new(vec![])
	}
}

/// A type to refer to a sequence of elements of the same type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Encode, Decode, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefSequence<F: Form = MetaForm> {
	/// The element type of the sequence type.
	#[serde(rename = "type")]
	type_param: F::TypeId,
}

impl IntoCompact for TypeDefSequence {
	type Output = TypeDefSequence<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefSequence {
			type_param: registry.register_type(&self.type_param),
		}
	}
}

impl TypeDefSequence {
	/// Creates a new sequence type.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(type_param: MetaType) -> Self {
		Self { type_param }
	}

	/// Creates a new sequence type.
	///
	/// Use this constructor if you want to instantiate from a given
	/// compile-time type.
	pub fn of<T>() -> Self
	where
		T: TypeInfo + 'static,
	{
		Self::new(MetaType::new::<T>())
	}
}
