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

use crate::tm_std::*;

use crate::{
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Metadata, Registry,
};
use serde::Serialize;

/// A field of a struct like data type.
///
/// Name is optional so it can represent both named and unnamed fields.
///
/// This can be a named field of a struct type or a struct variant.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct Field<F: Form = MetaForm> {
	/// The name of the field. None for unnamed fields.
	#[serde(skip_serializing_if = "Option::is_none")]
	name: Option<F::String>,
	/// The type of the field.
	#[serde(rename = "type")]
	ty: F::TypeId,
}

impl IntoCompact for Field {
	type Output = NamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		NamedField {
			name: self.name.map(|name| registry.register_string(name)),
			ty: registry.register_type(&self.ty),
		}
	}
}

impl Field {
	/// Creates a new field.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(name: Option<<MetaForm as Form>::String>, ty: MetaType) -> Self {
		Self { name, ty }
	}

	/// Creates a new named field
	pub fn named(name: <MetaForm as Form>::String, ty: MetaType) -> Self {
		Self::new(Some(name), ty)
	}

	/// Creates a new named field.
	///
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn named_of<T>(name: <MetaForm as Form>::String) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(Some(name), MetaType::new::<T>())
	}

	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate an unnamed field from a given meta type.
	pub fn unnamed(meta_type: MetaType) -> Self {
		Self::new(None, meta_type)
	}

	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate an unnamed field from a given compile-time
	/// type.
	pub fn unnamed_of<T>() -> Self
		where
			T: Metadata + ?Sized + 'static,
	{
		Self::new(None, MetaType::new::<T>())
	}
}

/// A composite type builder has no fields (e.g. a unit struct)
pub enum NoFields {}
/// A composite type builder only allows named fields (e.g. a struct)
pub enum NamedFields {}
/// A composite type builder only allows unnamed fields (e.g. a tuple)
pub enum UnnamedFields {}
