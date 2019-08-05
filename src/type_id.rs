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
	utils::is_rust_identifier,
	IntoCompact, MetaType, Metadata, Registry,
};
use derive_more::From;
use serde::Serialize;

/// Implementors return their meta type identifiers.
pub trait HasTypeId {
	/// Returns the static type identifier for `Self`.
	fn type_id() -> TypeId;
}

/// Represents the namespace of a type definition.
///
/// This consists of several segments that each have to be a valid Rust identifier.
/// The first segment represents the crate name in which the type has been defined.
///
/// Rust prelude type may have an empty namespace definition.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(transparent)]
pub struct Namespace<F: Form = MetaForm> {
	/// The segments of the namespace.
	segments: Vec<F::String>,
}

/// An error that may be encountered upon constructing namespaces.
#[derive(PartialEq, Eq, Debug)]
pub enum NamespaceError {
	/// If the module path does not at least have one segment.
	MissingSegments,
	/// If a segment within a module path is not a proper Rust identifier.
	InvalidIdentifier {
		/// The index of the errorneous segment.
		segment: usize,
	},
}

impl IntoCompact for Namespace {
	type Output = Namespace<CompactForm>;

	/// Compacts this namespace using the given registry.
	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		Namespace {
			segments: self
				.segments
				.into_iter()
				.map(|seg| registry.register_string(seg))
				.collect::<Vec<_>>(),
		}
	}
}

impl Namespace {
	/// Creates a new namespace from the given segments.
	pub fn new<S>(segments: S) -> Result<Self, NamespaceError>
	where
		S: IntoIterator<Item = <MetaForm as Form>::String>,
	{
		let segments = segments.into_iter().collect::<Vec<_>>();
		if segments.len() == 0 {
			return Err(NamespaceError::MissingSegments);
		}
		if let Some(err_at) = segments.iter().position(|seg| !is_rust_identifier(seg)) {
			return Err(NamespaceError::InvalidIdentifier { segment: err_at });
		}
		Ok(Self { segments })
	}

	/// Creates a new namespace from the given module path.
	///
	/// # Note
	///
	/// Module path is generally obtained from the `module_path!` Rust macro.
	pub fn from_str(module_path: <MetaForm as Form>::String) -> Result<Self, NamespaceError> {
		Self::new(module_path.split("::"))
	}

	/// Creates the prelude namespace.
	pub fn prelude() -> Self {
		Self { segments: vec![] }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "
	F::TypeId: Serialize,
	F::IndirectTypeId: Serialize
")]
#[serde(untagged)]
pub enum TypeId<F: Form = MetaForm> {
	Custom(TypeIdCustom<F>),
	Slice(TypeIdSlice<F>),
	Array(TypeIdArray<F>),
	Tuple(TypeIdTuple<F>),
	Primitive(TypeIdPrimitive),
}

impl IntoCompact for TypeId {
	type Output = TypeId<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeId::Custom(custom) => custom.into_compact(registry).into(),
			TypeId::Slice(slice) => slice.into_compact(registry).into(),
			TypeId::Array(array) => array.into_compact(registry).into(),
			TypeId::Tuple(tuple) => tuple.into_compact(registry).into(),
			TypeId::Primitive(primitive) => primitive.into(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TypeIdPrimitive {
	Bool,
	Char,
	Str,
	U8,
	U16,
	U32,
	U64,
	U128,
	I8,
	I16,
	I32,
	I64,
	I128,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeIdCustom<F: Form = MetaForm> {
	#[serde(rename = "custom.name")]
	name: F::String,
	#[serde(rename = "custom.namespace")]
	namespace: Namespace<F>,
	#[serde(rename = "custom.params")]
	type_params: Vec<F::TypeId>,
}

impl IntoCompact for TypeIdCustom {
	type Output = TypeIdCustom<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeIdCustom {
			name: registry.register_string(self.name),
			namespace: self.namespace.into_compact(registry),
			type_params: self
				.type_params
				.into_iter()
				.map(|param| registry.register_type(&param))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeIdCustom {
	pub fn new<T>(name: &'static str, namespace: Namespace, type_params: T) -> Self
	where
		T: IntoIterator<Item = MetaType>,
	{
		Self {
			name,
			namespace,
			type_params: type_params.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::IndirectTypeId: Serialize")]
pub struct TypeIdArray<F: Form = MetaForm> {
	#[serde(rename = "array.len")]
	pub len: u16,
	#[serde(rename = "array.type")]
	pub type_param: F::IndirectTypeId,
}

impl IntoCompact for TypeIdArray {
	type Output = TypeIdArray<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeIdArray {
			len: self.len,
			type_param: registry.register_type(&self.type_param),
		}
	}
}

impl TypeIdArray {
	pub fn new(len: u16, type_param: MetaType) -> Self {
		Self { len, type_param }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(transparent)]
pub struct TypeIdTuple<F: Form = MetaForm> {
	pub type_params: Vec<F::TypeId>,
}

impl IntoCompact for TypeIdTuple {
	type Output = TypeIdTuple<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeIdTuple {
			type_params: self
				.type_params
				.into_iter()
				.map(|param| registry.register_type(&param))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeIdTuple {
	pub fn new<T>(type_params: T) -> Self
	where
		T: IntoIterator<Item = MetaType>,
	{
		Self {
			type_params: type_params.into_iter().collect(),
		}
	}

	pub fn unit() -> Self {
		Self::new(vec![])
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::IndirectTypeId: Serialize")]
pub struct TypeIdSlice<F: Form = MetaForm> {
	#[serde(rename = "slice.type")]
	type_param: F::IndirectTypeId,
}

impl IntoCompact for TypeIdSlice {
	type Output = TypeIdSlice<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeIdSlice {
			type_param: registry.register_type(&self.type_param),
		}
	}
}

impl TypeIdSlice {
	pub fn new(type_param: MetaType) -> Self {
		Self { type_param }
	}

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
	fn namespace_from_str() {
		assert_eq!(
			Namespace::from_str("hello::world"),
			Ok(Namespace {
				segments: vec!["hello", "world"]
			})
		);
		assert_eq!(
			Namespace::from_str("::world"),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
	}
}
