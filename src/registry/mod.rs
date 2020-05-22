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
	meta_type::{MetaType, MetaTypeGeneric, MetaTypeParameterized},
	MetaTypeParameterValue, Type, InternedTypeId,
};
use interner::{Interner, UntrackedSymbol};
use derive_more::From;
use serde::Serialize;

pub mod interner;

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
	param_stack: Vec<MetaTypeParameterValue>,
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
	fn intern_type_id(&mut self, type_id: InternedTypeId) -> (bool, UntrackedSymbol<InternedTypeId>) {
		let (inserted, symbol) = self.type_table.intern_or_get(type_id);
		(inserted, symbol.into_untracked())
	}

	// todo: [AJ] combine with above private method?
	fn intern_type<F, T>(&mut self, type_id: InternedTypeId, f: F) -> UntrackedSymbol<InternedTypeId>
	where
		F: FnOnce() -> T,
		T: IntoCompact<Output = InternedType<CompactForm>>,
	{
		let (inserted, symbol) = self.intern_type_id(type_id);
		if inserted {
			let registry_type = f();
			let compact_id = registry_type.into_compact(self);
			self.types.insert(symbol.clone(), compact_id);
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
	pub fn register_type(&mut self, ty: &MetaType) -> UntrackedSymbol<InternedTypeId> {
		// let register_generic_type = |ty: &MetaTypeConcrete| {
		// 	self.intern_type(TypeId::Path(ty.path()), || {
		// 		RegistryType::Definition(TypeDef {
		// 			path: ty.path().into_compact(self),
		// 			ty: ty.type_info().into_compact(self),
		// 		})
		// 	})
		// };

		match ty {
			MetaType::Concrete(concrete) => {
				if !concrete.params.is_empty() {
					let generic_meta_type = MetaType::Generic(MetaTypeGeneric {
						fn_type_info: concrete.fn_type_info,
						path: concrete.path.clone(),
					});

					let generic: InternedGenericType<MetaForm> = InternedGenericType {
						ty: generic_meta_type,
						params: concrete
							.params
							.iter()
							.map(|p| MetaType::Concrete(p.concrete.clone()))
							.collect(),
					};

					let type_id = InternedTypeId::Generic(generic.clone().into_compact(self));

					self.intern_type(type_id, || InternedType::Generic(generic))
				} else {
					let type_id = InternedTypeId::Any(concrete.type_id);
					self.intern_type(type_id, || {
						let type_info = (concrete.fn_type_info)();
						InternedType::Definition(InternedTypeDef {
							path: concrete.path.clone(),
							ty: type_info,
						})
					})
				}
			}
			MetaType::Generic(ty) => {
				let type_id = InternedTypeId::Path(ty.path.clone());
				self.intern_type(type_id, || {
					let type_info = (ty.fn_type_info)();
					InternedType::Definition(InternedTypeDef {
						path: ty.path.clone(),
						ty: type_info,
					})
				})
			}
			MetaType::Parameter(p) => {
				let generic_meta_type = MetaType::Generic(p.parent.clone());
				let type_parameter = InternedTypeParameter {
					parent: generic_meta_type,
					name: p.name,
				};
				let param_type_id = InternedTypeId::Parameter(type_parameter.clone().into_compact(self));
				self.intern_type(param_type_id, || InternedType::Parameter(type_parameter))
			}
			MetaType::Parameterized(parameterized) => {
				let generic_meta_type = MetaType::Generic(MetaTypeGeneric {
					fn_type_info: parameterized.concrete.fn_type_info,
					path: parameterized.concrete.path.clone(),
				});

				self.param_stack.extend(parameterized.params.iter().cloned().rev());

				let params = parameterized
					.concrete
					.params
					.iter()
					.map(|concrete_param| {
						// todo: use Peekable api?
						if let Some(param) = self.param_stack.pop() {
							if param.concrete_type_id() == concrete_param.concrete.type_id {
								self.register_type(&param.into())
							} else if !concrete_param.concrete.params.is_empty() {
								self.param_stack.push(param);
								// recurse
								self.register_type(&MetaType::Parameterized(MetaTypeParameterized {
									concrete: concrete_param.concrete.clone(),
									params: Vec::new(),
								}))
							} else {
								panic!("Should either be matching concrete type (e.g. bool) or parameterized e.g. Option<T>")
							}
						} else {
							self.register_type(&&MetaType::Concrete(concrete_param.concrete.clone()))
						}
					})
					.collect::<Vec<_>>();

				let generic = InternedGenericType {
					ty: self.register_type(&generic_meta_type),
					params,
				};

				let type_id = InternedTypeId::Generic(generic.clone());

				self.intern_type(type_id, || InternedType::Generic(generic))
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

////////////////////////////////////////////

use crate::{
	form::{Form, MetaForm},
	Path,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum InternedType<F: Form = MetaForm> {
	/// The definition of the type
	Definition(InternedTypeDef<F>),
	/// The type is specified by a parameter of the parent type
	Parameter(InternedTypeParameter<F>),
	/// The type of the field is a generic type with the given type params
	Generic(InternedGenericType<F>),
}

impl IntoCompact for InternedType<MetaForm> {
	type Output = InternedType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			InternedType::Definition(definition) => definition.into_compact(registry).into(),
			InternedType::Parameter(parameter) => parameter.into_compact(registry).into(),
			InternedType::Generic(generic) => generic.into_compact(registry).into(),
		}
	}
}

impl IntoCompact for InternedType<CompactForm> {
	type Output = InternedType<CompactForm>;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct InternedTypeDef<F: Form = MetaForm> {
	#[serde(skip_serializing_if = "Path::is_empty")]
	path: Path<F>,
	ty: Type<F>,
}

impl IntoCompact for InternedTypeDef<MetaForm> {
	type Output = InternedTypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		InternedTypeDef {
			path: self.path.into_compact(registry),
			ty: self.ty.into_compact(registry),
		}
	}
}

/// A generic parameter of a parameterized MetaType.
///
/// e.g. the `T` in `Option<T>`
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct InternedTypeParameter<F: Form = MetaForm> {
	name: F::String,
	parent: F::Type,
}

impl IntoCompact for InternedTypeParameter<MetaForm> {
	type Output = InternedTypeParameter<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		InternedTypeParameter {
			name: registry.register_string(self.name),
			parent: registry.register_type(&self.parent),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct InternedGenericType<F: Form = MetaForm> {
	ty: F::Type, // this has to be the same for all instances of generic types
	params: Vec<F::Type>,
}

impl IntoCompact for InternedGenericType<MetaForm> {
	type Output = InternedGenericType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		InternedGenericType {
			ty: registry.register_type(&self.ty),
			params: registry.register_types(self.params),
		}
	}
}

impl IntoCompact for InternedGenericType<CompactForm> {
	type Output = InternedGenericType<CompactForm>;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self
	}
}

impl<F> InternedGenericType<F>
where
	F: Form,
{
	pub fn new<P>(ty: F::Type, params: P) -> Self
	where
		P: IntoIterator<Item = F::Type>,
	{
		InternedGenericType {
			ty,
			params: params.into_iter().collect(),
		}
	}
}
