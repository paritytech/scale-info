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
	IntoCompact, Field, Registry,
};
use derive_more::From;
use serde::Serialize;

/// A struct type, consisting of either named (struct) or unnamed (tuple struct) fields
///
/// # Examples
///
/// ## A Rust struct with named fields.
///
/// ```
/// struct Person {
///     name: String,
///     age_in_years: u8,
///     friends: Vec<Person>,
/// }
/// ```
///
/// ## A tuple struct with unnamed fields.
///
/// ```
/// struct Color(u8, u8, u8);
/// ```
///
/// ## A so-called unit struct
///
/// ```
/// struct JustAMarker;
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "lowercase")]
pub struct TypeStruct<F: Form = MetaForm> {
	fields: Vec<T>
}

impl<T> IntoCompact for TypeStruct<T> {
	type Output = TypeStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		registry.register_types(self.fields)
	}
}

impl<T> TypeStruct<T> {
	/// Creates a new struct definition with named fields.
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit() -> Self {
		Self { fields: vec![] }
	}
}

struct StructBuilder<F> {

}

impl<F> StructBuilder<F> {
	pub fn named_fields() {

	}

	pub fn unnamed_fields() {

	}
}
