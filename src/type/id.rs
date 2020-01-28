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
	IntoCompact, MetaType, Registry,
};
use serde::Serialize;

/// A type identifier for custom type definitions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeId<F: Form = MetaForm> {
	/// The name of the custom type.
	name: F::String,
	/// The namespace in which the custom type has been defined.
	namespace: Namespace<F>,
	/// The generic type parameters of the custom type in use.
	#[serde(rename = "params")]
	type_params: Vec<F::Type>,
}

impl IntoCompact for TypeId {
	type Output = TypeId<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeId {
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

impl TypeId {
	/// Creates a new type identifier to refer to a custom type definition.
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
		if segments.is_empty() {
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
	pub fn from_module_path(module_path: <MetaForm as Form>::String) -> Result<Self, NamespaceError> {
		Self::new(module_path.split("::"))
	}

	/// Creates the prelude namespace.
	pub fn prelude() -> Self {
		Self { segments: vec![] }
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
