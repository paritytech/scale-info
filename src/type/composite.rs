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
	fields::{Field, NoFields, NamedFields},
	form::{CompactForm, Form, MetaForm}, IntoCompact, Field, Path, Namespace, Registry
};
use derive_more::From;
use serde::Serialize;
use crate::fields::UnnamedFields;

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
	fields: Vec<T>,
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
	pub fn new(name: &'static str, namespace: Namespace) -> TypeCompositeBuilder
	{
		TypeCompositeBuilder::new(
			Self {
				path: Path::new(name, namespace, Vec::new()),
				fields: Vec::new(),
			}
		)
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit(path: Path) -> Self {
		Self::new(path).done()
	}
}

pub struct TypeCompositeBuilder<F = NoFields> {
	ty: TypeComposite,
	fields_marker: PhantomData<fn() -> F>,
}

impl TypeCompositeBuilder {
	pub fn new<F>(ty: TypeComposite) -> TypeCompositeBuilder<F> {
		Self {
			ty,
			fields_marker: Default::default()
		}
	}

	pub fn named_fields(self) -> TypeCompositeBuilder<NamedFields> {
		Self::new(self.ty)
	}

	pub fn unnamed_fields(self) -> TypeCompositeBuilder<UnnamedFields> {
		Self::new(self.ty)
	}

	// todo: [AJ] add type params (only allow on types with fields?)

	pub fn done(self) -> TypeComposite {
		self.ty
	}
}
