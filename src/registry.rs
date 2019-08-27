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
//! Types with the same name are uniquely identifiable by introducing namespaces.
//! For this the normal Rust namespace of a type is used where it has been defined it.
//! Rust prelude types live within the so-called root namespace that is just empty.
//! In general namespaces are ordered sequences of symbols and thus also profit from
//! string deduplication.

use crate::tm_std::*;
use crate::{
	form::CompactForm,
	interner::{Interner, UntrackedSymbol},
	meta_type::MetaType,
	TypeDef, TypeId,
};
use serde::Serialize;

/// Compacts the implementor using a registry.
pub trait IntoCompact {
	/// The compact version of `Self`.
	type Output;

	/// Compacts `self` by using the registry for caching and compaction.
	fn into_compact(self, registry: &mut Registry) -> Self::Output;
}

/// The pair of associated type identifier and structure.
///
/// This exists only as compactified version and is part of the registry.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TypeIdDef {
	/// The identifier of the type.
	id: TypeId<CompactForm>,
	/// The definition (aka internal structure) of the type.
	def: TypeDef<CompactForm>,
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
	type_table: Interner<AnyTypeId>,
	/// The database where registered types actually reside.
	///
	/// This is going to be serialized upon serlialization.
	#[serde(serialize_with = "serialize_registry_types")]
	types: BTreeMap<UntrackedSymbol<core::any::TypeId>, TypeIdDef>,
}

/// Serializes the types of the registry by removing their unique IDs
/// and instead serialize them in order of their removed unique ID.
fn serialize_registry_types<S>(
	types: &BTreeMap<UntrackedSymbol<core::any::TypeId>, TypeIdDef>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	let types = types.values().collect::<Vec<_>>();
	types.serialize(serializer)
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

	/// Registeres the given string into the registry and returns
	/// its respective associated string symbol.
	pub fn register_string(&mut self, string: &'static str) -> UntrackedSymbol<&'static str> {
		self.string_table.intern_or_get(string).1.into_untracked()
	}

	/// Registeres the given type ID into the registry.
	///
	/// Returns `false` as the first return value if the type ID has already
	/// been registered into this registry.
	/// Returns the associated type ID symbol as second return value.
	///
	/// # Note
	///
	/// This is an internal API and should not be called directly from the outside.
	fn intern_type_id(&mut self, any_type_id: AnyTypeId) -> (bool, UntrackedSymbol<AnyTypeId>) {
		let (inserted, symbol) = self.type_table.intern_or_get(any_type_id);
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
	pub fn register_type(&mut self, ty: &MetaType) -> UntrackedSymbol<AnyTypeId> {
		let (inserted, symbol) = self.intern_type_id(ty.any_id());
		if inserted {
			let compact_id = ty.type_id().into_compact(self);
			let compact_def = ty.type_def().into_compact(self);
			self.types.insert(
				symbol,
				TypeIdDef {
					id: compact_id,
					def: compact_def,
				},
			);
		}
		symbol
	}
}
