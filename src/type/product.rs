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

use crate::{NamedField, UnnamedField, form::{CompactForm, Form, MetaForm}, IntoCompact, Registry, TypePath};
use derive_more::From;
use serde::Serialize;

/// A Product type (consisting of fields) e.g. a struct
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum TypeProduct<F: Form = MetaForm> {
	/// A struct with named fields
	Struct(TypeProductStruct<F>),
	/// A tuple struct with unnamed fields
	TupleStruct(TypeProductTupleStruct<F>),
}

impl IntoCompact for TypeProduct {
	type Output = TypeProduct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeProduct::Struct(s) => {
				TypeProduct::Struct(s.into_compact(registry))
			},
			TypeProduct::TupleStruct(ts) => {
				TypeProduct::TupleStruct(ts.into_compact(registry))
			},
		}
	}
}

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
pub struct TypeProductStruct<F: Form = MetaForm> {
	/// The path of the struct
	path: TypePath<F>,
	/// The named fields of the struct.
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for TypeProductStruct {
	type Output = TypeProductStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeProductStruct {
//			fields: self
//				.fields
//				.into_iter()
//				.map(|field| field.into_compact(registry))
//				.collect::<Vec<_>>(),
			path: self.path.into_compact(registry),
			fields: registry.register_types(self.fields),
		}
	}
}

impl TypeProductStruct {
	/// Creates a new struct definition with named fields.
	pub fn new<F>(path: TypePath, fields: F) -> Self
		where
			F: IntoIterator<Item = NamedField>,
	{
		Self {
			path,
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
pub struct TypeProductTupleStruct<F: Form = MetaForm> {
	/// The path of the struct
	path: TypePath<F>,
	/// The unnamed fields.
	#[serde(rename = "types")]
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for TypeProductTupleStruct {
	type Output = TypeProductTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeProductTupleStruct {
			path: self.path.into_compact(registry),
			fields: registry.register_types(self.fields),
//			fields: self
//				.fields
//				.into_iter()
//				.map(|field| field.into_compact(registry))
//				.collect::<Vec<_>>(),
		}
	}
}

impl TypeProductTupleStruct {
	/// Creates a new tuple-struct.
	pub fn new<F>(path: TypePath, fields: F) -> Self
		where
			F: IntoIterator<Item = UnnamedField>,
	{
		Self {
			path,
			fields: fields.into_iter().collect(),
		}
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit(path: TypePath) -> Self {
		Self { path, fields: vec![] }
	}
}


