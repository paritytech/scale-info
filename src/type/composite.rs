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

use crate::{fields::{Fields}, form::{CompactForm, Form, MetaForm}, IntoCompact, Field, Path, Namespace, Registry, MetaType};
use derive_more::From;
use serde::Serialize;
use crate::fields::{UnnamedFields, FieldsBuilder};

/// A composite type, consisting of either named (struct) or unnamed (tuple struct) fields
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
pub struct TypeComposite<F: Form = MetaForm> {
	path: Path<F>,
	fields: Vec<Field<F>>,
}

impl<T> IntoCompact for TypeComposite<T> {
	type Output = TypeComposite<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeComposite {
			path: self.path.into_compact(),
			fields: registry.map_into_compact(self.fields),
		}
	}
}

impl<T> TypeComposite<T> {
	/// Creates a new struct definition with named fields.
	pub fn new(name: &'static str, namespace: Namespace) -> TypeCompositeBuilder {
		TypeCompositeBuilder::new(Self {
			path: Path::new(name, namespace, Vec::new()),
			fields: Vec::new(),
		})
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit(name: &'static str, namespace: Namespace) -> Self {
		Self::new(name, namespace).done()
	}
}

pub struct TypeCompositeBuilder {
	ty: TypeComposite,
	fields: Fields,
}

impl TypeCompositeBuilder {
	pub fn new<F>(ty: TypeComposite) -> TypeCompositeBuilder {
		Self {
			ty,
			fields: Fields::unit(),
		}
	}

	pub fn type_params<I>(self, type_params: I) -> Self
	where
		I: IntoIterator<Item = MetaType>
	{
		// todo: [AJ] difference between let mut this and "lens" style
		Self {
			ty: TypeComposite {
				path: Path {
					type_params: type_params.into_iter().collect(),
					..self.ty.path
				},
				..self.ty
			},
			..self
		}
	}

	pub fn fields(self, fields: FieldsBuilder) -> Self {
		let mut this = self;
		this.fields = fields.done();
		this
	}

	pub fn done(self) -> TypeComposite {
		self.ty
	}
}
