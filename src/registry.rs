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

//! The registry has the purpose to compactify types found in type definitions.
//!
//! This is done by deduplicating common types in order to reuse
//! their definitions which can grow arbitrarily large. A type is uniquely
//! identified by its type identifier that is therefore used to refer to types
//! and their definitions.
//!
//! Types with the same name are uniquely identifiable by introducing
//! namespaces. For this the normal Rust namespace of a type is used where it
//! has been defined. Rust prelude types live within the so-called root
//! namespace that is just empty.

use crate::tm_std::*;
use crate::{
	form::{CompactForm, Form},
	interner::{Interner, UntrackedSymbol},
	meta_type::MetaType,
	Type,
};
use scale::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Compacts the implementor using a registry.
pub trait IntoCompact {
	/// The compact version of `Self`.
	type Output;

	/// Compacts `self` by using the registry for caching and compaction.
	fn into_compact(self, registry: &mut Registry) -> Self::Output;
}

impl IntoCompact for &'static str {
	type Output = <CompactForm as Form>::String;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self.to_string()
	}
}

/// The registry for compaction of type identifiers and definitions.
///
/// The registry consists of a cache for already compactified type identifiers and definitions.
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
	/// The cache for already registered types.
	///
	/// This is just an accessor to the actual database
	/// for all types found in the `types` field.
	#[serde(skip)]
	type_table: Interner<TypeId>,
	/// The database where registered types actually reside.
	///
	/// This is going to be serialized upon serlialization.
	#[serde(serialize_with = "serialize_registry_types")]
	types: BTreeMap<UntrackedSymbol<core::any::TypeId>, Type<CompactForm>>,
}

/// Serializes the types of the registry by removing their unique IDs
/// and instead serialize them in order of their removed unique ID.
fn serialize_registry_types<S>(
	types: &BTreeMap<UntrackedSymbol<core::any::TypeId>, Type<CompactForm>>,
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

impl Encode for Registry {
	fn size_hint(&self) -> usize {
		mem::size_of::<u32>() + mem::size_of::<Type<CompactForm>>() * self.types.len()
	}

	fn encode_to<W: scale::Output>(&self, dest: &mut W) {
		if self.types.len() > u32::max_value() as usize {
			panic!("Attempted to encode too many elements.");
		}
		scale::Compact(self.types.len() as u32).encode_to(dest);

		for ty in self.types.values() {
			ty.encode_to(dest);
		}
	}
}

impl Registry {
	/// Creates a new empty registry.
	pub fn new() -> Self {
		Self {
			type_table: Interner::new(),
			types: BTreeMap::new(),
		}
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
		let (inserted, symbol) = self.intern_type_id(ty.type_id());
		if inserted {
			let compact_id = ty.type_info().into_compact(self);
			self.types.insert(symbol, compact_id);
		}
		symbol
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

/// A read-only registry, to be used for decoding/deserializing
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Decode)]
pub struct RegistryReadOnly {
	types: Vec<Type<CompactForm>>,
}

impl From<Registry> for RegistryReadOnly {
	fn from(registry: Registry) -> Self {
		RegistryReadOnly {
			types: registry.types.values().cloned().collect::<Vec<_>>()
		}
	}
}

impl RegistryReadOnly {
	/// Returns the type definition for the given identifier, `None` if no type found for that ID.
	pub fn resolve(&self, id: NonZeroU32) -> Option<&Type<CompactForm>> {
		self.types.get((id.get() - 1) as usize)
	}
}
