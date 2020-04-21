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

//! The registry has the purpose to compactify types and strings found in type
//! definitions and identifiers such as symbol names.
//!
//! This is done by deduplicating common strings and types in order to reuse
//! their definitions which can grow arbitrarily large. A type is uniquely
//! identified by its type identifier that is therefore used to refer to types
//! and their definitions.
//!
//! Since symbol names etc. are often shared between different types they are
//! as well deduplicated.
//!
//! Types with the same name are uniquely identifiable by introducing
//! namespaces. For this the normal Rust namespace of a type is used where it
//! has been defined it. Rust prelude types live within the so-called root
//! namespace that is just empty. In general namespaces are ordered sequences of
//! symbols and thus also profit from string deduplication.

use crate::tm_std::*;
use crate::{form::CompactForm, interner::{Interner, UntrackedSymbol}, meta_type::MetaType, Type, TypeId};
use crate::meta_type::MetaTypeKind;
use derive_more::From;
use serde::Serialize;

/// Compacts the implementor using a registry.
pub trait IntoCompact {
	/// The compact version of `Self`.
	type Output;

	/// Compacts `self` by using the registry for caching and compaction.
	fn into_compact(self, registry: &mut Registry) -> Self::Output;
}

/// The registry for compaction of type identifiers and definitions.
///
/// The registry consists of a cache for strings such as symbol names
/// and a cache for already compactified type identifiers and definitions.
///
/// Whenever using the registry to compact a type all of its sub-types
/// are going to be registered recursively as well. A type is a sub-type
/// of another type if it is used by its identifier or structure.
///
/// # Note
///
/// A type can be a sub-type of itself. In this case the registry has a builtin
/// mechanism to stop recursion before going into an infinite loop.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Registry {
	/// The cache for already registered strings.
	#[serde(rename = "strings")]
	string_table: Interner<&'static str>,
	/// The cache for already registered types.
	///
	/// This is just an accessor to the actual database
	/// for all types found in the `types` field.
	#[serde(skip)]
	type_table: Interner<TypeId>,
	/// The database where registered types actually reside.
	///
	/// This is going to be serialized upon serialization.
	#[serde(serialize_with = "serialize_registry_types")]
	types: BTreeMap<UntrackedSymbol<core::any::TypeId>, RegistryType<CompactForm>>,
}

/// Serializes the types of the registry by removing their unique IDs
/// and instead serialize them in order of their removed unique ID.
fn serialize_registry_types<S>(
	types: &BTreeMap<UntrackedSymbol<core::any::TypeId>, RegistryType<CompactForm>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	let types = types.values().collect::<Vec<_>>();
	types.serialize(serializer)
}

impl Default for Registry {
	fn default() -> Self {
		Self::new()
	}
}

impl Registry {
	/// Creates a new empty registry.
	pub fn new() -> Self {
		Self {
			string_table: Interner::new(),
			type_table: Interner::new(),
			types: BTreeMap::new(),
		}
	}

	/// Registers the given string into the registry and returns
	/// its respective associated string symbol.
	pub fn register_string(&mut self, string: &'static str) -> UntrackedSymbol<&'static str> {
		self.string_table.intern_or_get(string).1.into_untracked()
	}

	/// Registers the given type ID into the registry.
	///
	/// Returns `false` as the first return value if the type ID has already
	/// been registered into this registry.
	/// Returns the associated type ID symbol as second return value.
	///
	/// # Note
	///
	/// This is an internal API and should not be called directly from the
	/// outside.
	fn intern_type_id(&mut self, type_id: TypeId) -> (bool, UntrackedSymbol<TypeId>) {
		let (inserted, symbol) = self.type_table.intern_or_get(type_id);
		(inserted, symbol.into_untracked())
	}

	/// Registers the given type into the registry and returns
	/// its associated type ID symbol.
	///
	/// # Note
	///
	/// Due to safety requirements the returns type ID symbol cannot
	/// be used later to resolve back to the associated type definition.
	/// However, since this facility is going to be used for serialization
	/// purposes this functionality isn't needed anyway.
	pub fn register_type(&mut self, ty: &MetaType) -> UntrackedSymbol<TypeId> {
		// todo: if any params register generic first
		match ty.kind() {
			MetaTypeKind::Concrete => {
				let any_type_id = TypeId::Any(ty.type_id());
				let generic_type_id = TypeId::Path(ty.path());
				// It's a generic type
				if ty.is_generic() {
					let (inserted, symbol) = self.intern_type_id(generic_type_id);
					if inserted {
						let registry_type = RegistryType::Definition(TypeDef {
							path: ty.path(),
							params: ty.params(),
							ty: ty.type_info(),
						});
						let compact_id = registry_type.into_compact(self);
						self.types.insert(symbol, compact_id);
					}
					symbol
				} else {
					let (inserted, symbol) = self.intern_type_id(any_type_id);
					if inserted {
						let generic_type = RegistryType::Definition(TypeDef {
							path: ty.path(),
							params: ty.params(),
							ty: ty.type_info(),
						});
						let compact_id = generic_type.into_compact(self);
						self.types.insert(symbol, compact_id);
					}
					let (inserted, symbol) = self.intern_type_id(any_type_id);
					if inserted {
						let generic_type = RegistryType::Generic(GenericType {
							ty: symbol,
							params: ty.params(),
						});
						let compact_id = generic_type.into_compact(self);
						self.types.insert(symbol, compact_id);
					}
					symbol
				}
			},
			MetaType::Parameter(parameter) => {
				todo!()
			},
		}
	}

	/// Calls `register_type` for each `MetaType` in the given `iter`
	pub fn register_types<I>(&mut self, iter: I) -> Vec<UntrackedSymbol<TypeId>>
	where
		I: IntoIterator<Item = MetaType>,
	{
		iter.into_iter().map(|i| self.register_type(&i)).collect::<Vec<_>>()
	}

	/// Converts an iterator into a Vec of the equivalent compact
	/// representations
	pub fn map_into_compact<I, T>(&mut self, iter: I) -> Vec<T::Output>
	where
		I: IntoIterator<Item = T>,
		T: IntoCompact,
	{
		iter.into_iter().map(|i| i.into_compact(self)).collect::<Vec<_>>()
	}
}

////////////////////////////////////////////

use crate::{
	form::{Form, MetaForm},
	Path,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub enum RegistryType<F: Form = MetaForm> {
	/// The definition of the type
	Definition(TypeDef<F>),
	/// The type is specified by a parameter of the parent type
	Parameter(TypeParameter<F>),
	/// The type of the field is a generic type with the given type params
	Generic(GenericType<F>),
}

impl IntoCompact for RegistryType {
	type Output = RegistryType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			Type::Concrete(ref ty) => ty.into_compact(registry).into(),
			Type::Parameter(ref param) => param.into_compact(registry).into(),
			Type::Generic(generic) => generic.into_compact(registry).into(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDef<F: Form = MetaForm> {
	path: Path<F>,
	params: Vec<TypeParameter<F>>, // points back to RegistryType::Parameter
	ty: Type<F>,
}

impl IntoCompact for TypeDef {
	type Output = TypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDef {
			path: self.path.into_compact(registry),
			params: self.registry.map_into_compact(params),
			ty: self.ty.into_compact(registry),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeParameter<F: Form = MetaForm> {
	name: F::String,
	// ty: F::Type,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct GenericType<F: Form = CompactForm> {
	ty: F::Type, // this has to be the same for all instances of generic types
	params: Vec<F::Type>,
}

impl IntoCompact for GenericType {
	type Output = GenericType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		GenericType {
			ty: registry.register_type(&self.ty),
			params: registry.register_types(self.params),
		}
	}
}

impl GenericType {
	pub fn new<P>(ty: Type, params: P) -> Self
	where
		P: IntoIterator<Item = <MetaForm as Form>::Type>
	{
		GenericType {
			ty,
			params: params.into_iter().collect(),
		}
	}
}
