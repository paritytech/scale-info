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
	interner::{StringInterner, StringSymbol, TypeIdInterner, TypeIdSymbol, UntrackedTypeIdSymbol},
	HasTypeId, Metadata, Namespace, TypeDef, TypeId,
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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TypeIdDef {
	id: TypeId,
	def: TypeDef,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Registry {
	string_table: StringInterner,
	typeid_table: TypeIdInterner,
	typedefs: Vec<TypeDef>,
}

impl Registry {
	pub fn register_name(&mut self, name: &'static str) -> StringSymbol {
		unimplemented!()
	}

	pub fn register_namespace(&mut self, namespace: Namespace) -> CompactNamespace {
		unimplemented!()
	}

	pub fn register_type<T>(&mut self) -> TypeIdSymbol
	where
		T: Metadata,
	{
		unimplemented!()
	}
}
