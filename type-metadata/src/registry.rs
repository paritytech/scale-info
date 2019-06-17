// Copyright 2019 Centrality Investments Limited
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

use std::collections::BTreeMap;

use crate::{
	interner::{StringInterner, TypeIdInterner},
	Metadata, TypeDef, TypeId,
};

pub struct Tables {
	pub string_table: StringInterner,
	pub typeid_table: TypeIdInterner,
}

/// Used by the type registry in order to recursively traverse through
/// all static generic types given a concrete metadata type.
pub trait RegisterSubtypes {
	/// Registers all subtypes for `Self`.
	///
	/// # Note
	///
	/// A subtype in this context is basically all types that make up any
	/// given generic types. E.g. `Option<T>` has `T` as subtype.
	fn register_subtypes(_registry: &mut Registry) {}
}

impl Tables {
	pub fn new() -> Self {
		Self {
			string_table: StringInterner::new(),
			typeid_table: TypeIdInterner::new(),
		}
	}
}

pub struct Registry<'t> {
	tables: &'t mut Tables,
	types: BTreeMap<TypeId, TypeDef>,
}

impl<'t> Registry<'t> {
	pub fn new(tables: &'t mut Tables) -> Self {
		Self {
			tables,
			types: BTreeMap::new(),
		}
	}

	pub fn register_type<T: Metadata>(&mut self) {
		let type_id = T::type_id();

		match type_id {
			TypeId::Primitive(_) => (),
			TypeId::Array(_) | TypeId::Slice(_) | TypeId::Tuple(_) => {
				T::register_subtypes(self);
			}
			TypeId::Custom(_) => {
				if !self.types.contains_key(&type_id) {
					self.types.insert(type_id.clone(), TypeDef::builtin());
					T::register_subtypes(self);
					let type_def = T::type_def();
					self.types.insert(type_id, type_def);
				}
			}
		}
	}
}
