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
use crate::{
	form::CompactForm,
	meta_type::{MetaType, MetaTypeDefinition, MetaTypeKind},
};
use interner::{Interner, UntrackedSymbol};
use serde::Serialize;

mod interned_type;
pub mod interner;

pub use interned_type::{InternedGenericType, InternedType, InternedTypeId, InternedTypeParameter};

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
	type_table: Interner<InternedTypeId>,
	/// Scope stack for resolving nested parameterized types
	#[serde(skip)]
	param_stack: Vec<MetaType>,
	/// The database where registered types actually reside.
	///
	/// This is going to be serialized upon serialization.
	#[serde(serialize_with = "serialize_registry_types")]
	types: BTreeMap<UntrackedSymbol<InternedTypeId>, InternedType<CompactForm>>,
}

/// Serializes the types of the registry by removing their unique IDs
/// and instead serialize them in order of their removed unique ID.
fn serialize_registry_types<S>(
	types: &BTreeMap<UntrackedSymbol<InternedTypeId>, InternedType<CompactForm>>,
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
	fn intern_type<F, T>(&mut self, type_id: InternedTypeId, f: F) -> UntrackedSymbol<InternedTypeId>
	where
		F: FnOnce() -> T,
		T: IntoCompact<Output = InternedType<CompactForm>>,
	{
		let (inserted, symbol) = self.type_table.intern_or_get(type_id);
		let symbol = symbol.into_untracked();
		if inserted {
			let registry_type = f();
			let compact_id = registry_type.into_compact(self);
			self.types.insert(symbol.clone(), compact_id);
		}
		symbol
	}

	fn intern_generic_type<P>(&mut self, type_def: &MetaTypeDefinition, params: P) -> UntrackedSymbol<InternedTypeId>
	where
		P: IntoIterator<Item = UntrackedSymbol<InternedTypeId>>,
	{
		let generic_ty = self.register_generic_type(type_def);
		let interned_generic = InternedGenericType::new(generic_ty, params);
		let type_id = InternedTypeId::Generic(interned_generic.clone());

		self.intern_type(type_id, || InternedType::<CompactForm>::Generic(interned_generic))
	}

	fn register_parameterized_type<'a, I>(
		&mut self,
		parameterized: &MetaType,
		parameter_values: I,
	) -> UntrackedSymbol<InternedTypeId>
	where
		I: IntoIterator<Item = &'a MetaType>,
		<I as IntoIterator>::IntoIter: DoubleEndedIterator,
	{
		self.param_stack.extend(parameter_values.into_iter().cloned().rev());

		let params = parameterized
			.params()
			.map(|concrete_param| {
				let mut peekable = self.param_stack.iter().peekable();
				if let Some(param) = peekable.peek() {
					if *param == concrete_param {
						let param = self.param_stack.pop().expect("parameter was peeked first");
						self.register_type(&param)
					} else if concrete_param.has_params() {
						// recurse
						self.register_parameterized_type(&concrete_param, Vec::new())
					} else {
						panic!(
							"Parameter {:?} should match the concrete type {:?} or be parameterized e.g. Option<T>",
							param.concrete_type_name(),
							concrete_param.concrete_type_name()
						);
					}
				} else {
					self.register_type(&concrete_param)
				}
			})
			.collect::<Vec<_>>();

		self.intern_generic_type(parameterized.type_def(), params)
	}

	fn register_generic_type(&mut self, ty: &MetaTypeDefinition) -> UntrackedSymbol<InternedTypeId> {
		let type_id = InternedTypeId::Path(ty.path());
		self.intern_type(type_id, || InternedType::definition(ty.path(), ty.type_info()))
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
	pub fn register_type(&mut self, ty: &MetaType) -> UntrackedSymbol<InternedTypeId> {
		match ty.kind() {
			MetaTypeKind::Concrete => {
				if ty.has_params() {
					let params = ty.params().map(|p| self.register_type(p)).collect::<Vec<_>>();
					self.intern_generic_type(ty.type_def(), params)
				} else {
					// The concrete type definition has no type parameters, so is not a generic type
					let type_id = InternedTypeId::Concrete(ty.concrete_type_id());
					self.intern_type(type_id, || {
						let type_info = ty.type_info();
						InternedType::definition(ty.path().clone(), type_info)
					})
				}
			}
			MetaTypeKind::Parameterized(param_values) => self.register_parameterized_type(ty, param_values),
			MetaTypeKind::Parameter(p) => {
				let parent = self.register_generic_type(&p.parent);
				let type_parameter = InternedTypeParameter::new(p.name, parent).into_compact(self);
				let param_type_id = InternedTypeId::Parameter(type_parameter.clone());
				self.intern_type(param_type_id, || InternedType::Parameter(type_parameter))
			}
		}
	}

	/// Calls `register_type` for each `MetaType` in the given `iter`
	pub fn register_types<I>(&mut self, iter: I) -> Vec<UntrackedSymbol<InternedTypeId>>
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
