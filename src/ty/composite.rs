// Copyright 2019-2020
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
	form::{CompactForm, Form, MetaForm}, state,
	Field, FieldsBuilder, IntoCompact, MetaType, Path, PathError, Registry,
};
use derive_more::From;
use serde::Serialize;

/// A composite type, consisting of either named (struct) or unnamed (tuple
/// struct) fields
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
	#[serde(skip_serializing_if = "Path::is_empty")]
	path: Path<F>,
	#[serde(rename = "params", skip_serializing_if = "Vec::is_empty")]
	type_params: Vec<F::TypeId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	fields: Vec<Field<F>>,
}

impl IntoCompact for TypeComposite {
	type Output = TypeComposite<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeComposite {
			path: self.path.into_compact(registry),
			type_params: registry.register_types(self.type_params),
			fields: registry.map_into_compact(self.fields),
		}
	}
}

impl TypeComposite {
	/// Creates a new struct definition with named fields.
	#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_ret_no_self))]
	pub fn new() -> TypeCompositeBuilder {
		TypeCompositeBuilder::default()
	}
}

pub struct TypeCompositeBuilder<S = state::PathNotAssigned> {
	path: Option<Path>,
	type_params: Vec<MetaType>,
	marker: PhantomData<fn() -> S>,
}

impl<S> Default for TypeCompositeBuilder<S> {
	fn default() -> Self {
		TypeCompositeBuilder {
			path: Default::default(),
			type_params: Default::default(),
			marker: Default::default(),
		}
	}
}

impl TypeCompositeBuilder<state::PathNotAssigned> {
	/// Set the Path for the type
	///
	/// # Panics
	///
	/// If the Path is invalid
	pub fn path(self, path: Result<Path, PathError>) -> TypeCompositeBuilder<state::PathAssigned> {
		TypeCompositeBuilder {
			path: Some(path.expect("Invalid Path")),
			type_params: self.type_params,
			marker: Default::default(),
		}
	}
}

impl TypeCompositeBuilder<state::PathAssigned> {
	fn build(self, fields: Vec<Field<MetaForm>>) -> TypeComposite {
		TypeComposite {
			path: self.path.expect("Path is assigned"),
			type_params: self.type_params,
			fields,
		}
	}

	pub fn fields<F>(self, fields: FieldsBuilder<F>) -> TypeComposite {
		self.build(fields.done())
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit(self) -> TypeComposite {
		self.build(Vec::new())
	}
}

impl<S> TypeCompositeBuilder<S> {
	pub fn type_params<I>(self, type_params: I) -> Self
	where
		I: IntoIterator<Item = MetaType>,
	{
		let mut this = self;
		this.type_params = type_params.into_iter().collect();
		this
	}
}
