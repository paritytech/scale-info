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
	/// Scope stack for resolving nested parameterized types
	#[serde(skip)]
	param_stack: Vec<UntrackedSymbol<TypeId>>,
	/// The database where registered types actually reside.
	///
	/// This is going to be serialized upon serialization.
	#[serde(serialize_with = "serialize_registry_types")]
	types: BTreeMap<UntrackedSymbol<TypeId>, RegistryType<CompactForm>>,
}

/// Serializes the types of the registry by removing their unique IDs
/// and instead serialize them in order of their removed unique ID.
fn serialize_registry_types<S>(
	types: &BTreeMap<UntrackedSymbol<TypeId>, RegistryType<CompactForm>>,
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
			param_stack: Vec::new(),
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

	// todo: [AJ] combine with above private method?
	fn intern_type<F>(&mut self, type_id: TypeId, f: F) -> UntrackedSymbol<TypeId>
	where
		F: FnOnce () -> RegistryType
	{
		let (inserted, symbol) = self.intern_type_id(type_id);
		if inserted {
			let registry_type = f();
			let compact_id = registry_type.into_compact(self);
			self.types.insert(symbol, compact_id);
		}
		symbol
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
		let any_type_id = TypeId::Any(ty.type_id());
		let generic_type_id = TypeId::Path(ty.path());


		match ty.kind() {
			MetaTypeKind::Generic => {
				// from MetaType::parameterized
				// todo: [AJ] need to resolve the id of the parameterized instance...

				let generic_params = self.register_types(ty.params().iter().map(|tp| {
					let parent = ty.clone();
					MetaType::parameter(tp.name, parent);
				}));

				unimplemented!()
				// let (inserted, symbol) = self.intern_type_id(generic_type_id);
				// if inserted {
				// 	let registry_type = RegistryType::Definition(TypeDef {
				// 		path: ty.path().into_compact(self),
				// 		params: self.map_into_compact(ty.params()),
				// 		ty: ty.type_info().into_compact(self),
				// 	});
				// 	let compact_id = registry_type.into_compact(self);
				// 	self.types.insert(symbol, compact_id);
				// }
				// symbol
			}
			MetaTypeKind::Concrete => {

				// todo: we know that we have fully concrete TPs here, so we set the params at the
				// top level (using any::TypeId) and then they are available as we walk down the tree
				// of types, where we can match a parameter type to this generic type parameter

				// It's a concrete instance of a generic type e.g. Option<bool>
				if ty.is_generic() {
					let generic_params = self.register_types(ty.params().iter().map(|tp| {
						let parent = ty.clone();
						MetaType::parameter(tp.name, parent);
					}));

					// PARAM STACK
					// let params = self.register_types(ty.params());
					// // push the type parameters onto the parameter stack
					// self.param_stack.extend_from_slice(&params);

					// register the generic definition
					let generic_symbol = self.intern_type(generic_type_id, || {
						RegistryType::Definition(TypeDef {
							path: ty.path().into_compact(self),
							params: generic_params,
							ty: ty.type_info().into_compact(self),
						})
					});

					let instance_params =
						self.register_types(ty.params().iter().map(|tp| { &tp.ty }));

					// register the generic instance
					let symbol = self.intern_type(any_type_id, || {
						RegistryType::Generic(GenericType {
							ty: generic_symbol,
							params: instance_params,
						})
					});
					symbol
				} else {
					// just a regular concrete type with no parameters
					self.intern_type(any_type_id, || {
						RegistryType::Definition(TypeDef {
							path: ty.path().into_compact(self),
							params: Vec::new(),
							ty: ty.type_info().into_compact(self),
						})
					});
				}
			},
			MetaTypeKind::Parameter(param_name, parent_type) => {
				// e.g. `a: T`
				let type_id = TypeId::Parameter(parent_type.path(), param_name);
				self.intern_type(type_id, || {
					RegistryType::Parameter(
						TypeParameter {
							name: *param_name,
							path: parent_type.path().into_compact(self),
						}
					)
				})
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
use std::collections::VecDeque;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub enum RegistryType<F: Form = CompactForm> {
	/// The definition of the type
	Definition(TypeDef<F>),
	/// The type is specified by a parameter of the parent type
	Parameter(TypeParameter<F>),
	/// The type of the field is a generic type with the given type params
	Generic(GenericType<F>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDef<F: Form = CompactForm> {
	path: Path<F>,
	params: Vec<F::Type>,
	ty: Type<F>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeParameter<F: Form = CompactForm> {
	name: F::String,
	path: Path<F>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct GenericType<F: Form = CompactForm> {
	ty: F::Type, // this has to be the same for all instances of generic types
	params: Vec<F::Type>,
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
