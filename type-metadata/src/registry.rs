// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of type-metadata.
//
// type-metadata is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// type-metadata is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with type-metadata.  If not, see <http://www.gnu.org/licenses/>.

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
