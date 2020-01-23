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
	NamedField, UnnamedField,
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Metadata, Registry,
};
use derive_more::From;
use serde::Serialize;

/// A Rust struct with named fields.
///
/// # Example
///
/// ```
/// struct Person {
///     name: String,
///     age_in_years: u8,
///     friends: Vec<Person>,
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDefStruct<F: Form = MetaForm> {
	/// The named fields of the struct.
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for TypeDefStruct {
	type Output = TypeDefStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefStruct {
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefStruct {
	/// Creates a new struct definition with named fields.
	pub fn new<F>(fields: F) -> Self
		where
			F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}
}



/// A tuple struct with unnamed fields.
///
/// # Example
///
/// ```
/// struct Color(u8, u8, u8);
/// ```
/// or a so-called unit struct
/// ```
/// struct JustAMarker;
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDefTupleStruct<F: Form = MetaForm> {
	/// The unnamed fields.
	#[serde(rename = "types")]
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for TypeDefTupleStruct {
	type Output = TypeDefTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefTupleStruct {
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefTupleStruct {
	/// Creates a new tuple-struct.
	pub fn new<F>(fields: F) -> Self
		where
			F: IntoIterator<Item = UnnamedField>,
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


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub enum TypeProduct<F: Form = MetaForm> {
	Struct(Vec<NamedField<F>>),
	TupleStruct()
}


