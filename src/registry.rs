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

use crate::{
	form::CompactForm,
	interner::{StringInterner, TypeIdInterner, UntrackedStringSymbol, UntrackedTypeIdSymbol},
	HasTypeId, Metadata, TypeDef, TypeId,
};
use serde::Serialize;

/// Used by the type registry in order to recursively traverse through
/// all static generic types given a concrete metadata type.
///
/// # Note
///
/// - Users should generally avoid implementing this manually and instead
///   rely on the automated implementation through the derive macro.
/// - The set of subtypes in this context consists of all types that make
///   up a concrete instance of `Self`. E.g. for the tuple type of
///   `(Vec<i32>, Box<u64>)` the direct subtypes are `Vec<i32>` and `Box<u64>`.
/// - For enums this has to enumerate all subtypes of all variants.
pub trait RegisterSubtypes {
	/// Registers all subtypes for `Self`.
	fn register_subtypes(_registry: &mut Registry) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntoCompactError {
	MissingTypeId {
		id: TypeId,
	},
	MissingString {
		string: &'static str,
	}
}

impl IntoCompactError {
	pub fn missing_typeid(type_id: &TypeId) -> Self {
		IntoCompactError::MissingTypeId {
			id: type_id.clone(),
		}
	}

	pub fn missing_string(string: &'static str) -> Self {
		IntoCompactError::MissingString { string }
	}
}

pub trait IntoCompact {
	type Output;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError>;
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TypeIdDef {
	id: TypeId<CompactForm>,
	def: TypeDef<CompactForm>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Registry {
	string_table: StringInterner,
	typeid_table: TypeIdInterner,
	types: Vec<TypeIdDef>,
}

/// Used by `IntoCompact` implementers.
impl Registry {
	pub fn register_string(&mut self, string: &'static str) -> UntrackedStringSymbol {
		self.string_table
			.intern_or_get(string)
			.1
			.into_untracked()
	}

	pub fn resolve_type_id(&self, type_id: &TypeId) -> Result<UntrackedTypeIdSymbol, IntoCompactError> {
		self.typeid_table
			.get(type_id)
			.ok_or(IntoCompactError::missing_typeid(type_id))
			.map(|symbol| symbol.into_untracked())
	}

	fn intern_typeid<T>(&mut self) -> (bool, UntrackedTypeIdSymbol)
	where
		T: ?Sized + HasTypeId,
	{
		let (inserted, symbol) = self.typeid_table.intern_or_get(T::type_id());
		(inserted, symbol.into_untracked())
	}

	pub fn register_type<T>(&mut self) -> UntrackedTypeIdSymbol
	where
		T: ?Sized + Metadata,
	{
		let (inserted, symbol) = self.intern_typeid::<T>();
		if inserted {
			T::register_subtypes(self);
			let compact_id = T::type_id().into_compact(self)
				.expect("the type identifier is expected to be registered at this point");
			let compact_def = T::type_def().into_compact(self)
				.expect("the type definition is expected to be registered at this point");
			self.types.push(TypeIdDef {
				id: compact_id,
				def: compact_def,
			});
		}
		symbol
	}
}
