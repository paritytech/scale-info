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

/// A named field.
///
/// This can be a named field of a struct type or a struct variant.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct NamedField<F: Form = MetaForm> {
	/// The name of the field.
	name: F::String,
	/// The type of the field.
	#[serde(rename = "type")]
	ty: F::Type,
}

impl IntoCompact for NamedField {
	type Output = NamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		NamedField {
			name: registry.register_string(self.name),
			ty: registry.register_type(&self.ty),
		}
	}
}

impl NamedField {
	/// Creates a new named field.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(name: <MetaForm as Form>::String, ty: MetaType) -> Self {
		Self { name, ty }
	}

	/// Creates a new named field.
	///
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn of<T>(name: <MetaForm as Form>::String) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(name, MetaType::new::<T>())
	}
}

/// An unnamed field from either a tuple-struct type or a tuple-struct variant.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
#[serde(transparent)]
pub struct UnnamedField<F: Form = MetaForm> {
	/// The type of the unnamed field.
	#[serde(rename = "type")]
	ty: F::Type,
}

impl IntoCompact for UnnamedField {
	type Output = UnnamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		UnnamedField {
			ty: registry.register_type(&self.ty),
		}
	}
}

impl UnnamedField {
	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(meta_type: MetaType) -> Self {
		Self { ty: meta_type }
	}

	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn of<T>() -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(MetaType::new::<T>())
	}
}
