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
	Field, FieldsBuilder, IntoCompact, MetaType, Namespace, Path, PathBuilder, Registry,
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
	#[serde(flatten)]
	path: Path<F>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	fields: Vec<Field<F>>,
}

impl IntoCompact for TypeComposite {
	type Output = TypeComposite<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeComposite {
			path: self.path.into_compact(registry),
			fields: registry.map_into_compact(self.fields),
		}
	}
}

impl TypeComposite {
	/// Creates a new struct definition with named fields.
	#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_ret_no_self))]
	pub fn new(name: &'static str, namespace: Namespace) -> TypeCompositeBuilder {
		TypeCompositeBuilder {
			path: Path::new(name, namespace),
		}
	}
}

pub struct TypeCompositeBuilder {
	path: PathBuilder,
}

impl TypeCompositeBuilder {
	pub fn type_params<I>(self, type_params: I) -> Self
	where
		I: IntoIterator<Item = MetaType>,
	{
		let mut this = self;
		this.path.type_params(type_params);
		this
	}

	pub fn fields<F>(self, fields: FieldsBuilder<F>) -> TypeComposite {
		TypeComposite {
			path: self.path.done(),
			fields: fields.done(),
		}
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit(self) -> TypeComposite {
		TypeComposite {
			path: self.path.done(),
			fields: Vec::new(),
		}
	}
}
