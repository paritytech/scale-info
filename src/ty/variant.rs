// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
	build::FieldsBuilder,
	form::{CompactForm, Form, MetaForm},
	Field, IntoCompact, Registry,
};
use derive_more::From;
use serde::Serialize;

/// A Enum type (consisting of variants).
///
/// # Examples
///
/// ## A Rust enum, aka tagged union.
///
/// ```
/// enum MyEnum {
///     RustAllowsForClikeVariants,
///     AndAlsoForTupleStructs(i32, bool),
///     OrStructs {
///         with: i32,
///         named: bool,
///         fields: [u8; 32],
///     },
///     ItIsntPossibleToSetADiscriminantThough,
/// }
/// ```
///
/// ## A C-like enum type.
///
/// ```
/// enum Days {
///     Monday,
///     Tuesday,
///     Wednesday,
///     Thursday = 42, // Also allows to manually set the discriminant!
///     Friday,
///     Saturday,
///     Sunday,
/// }
/// ```
///
/// ## An empty enum (for marker purposes)
///
/// ```
/// enum JustAMarker {}
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "lowercase")]
pub struct TypeDefVariant<F: Form = MetaForm> {
	#[serde(skip_serializing_if = "Vec::is_empty")]
	variants: Vec<Variant<F>>,
}

impl IntoCompact for TypeDefVariant {
	type Output = TypeDefVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefVariant {
			variants: registry.map_into_compact(self.variants),
		}
	}
}

impl TypeDefVariant {
	/// Create a new `TypeDefVariant` with the given variants
	pub fn new<I>(variants: I) -> Self
	where
		I: IntoIterator<Item = Variant>,
	{
		Self {
			variants: variants.into_iter().collect(),
		}
	}
}

/// A struct enum variant with either named (struct) or unnamed (tuple struct)
/// fields.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
/// //  ^^^^ this is a unit struct enum variant
///     Add(i32, i32),
/// //  ^^^^^^^^^^^^^ this is a tuple-struct enum variant
///     Minus { source: i32 }
/// //  ^^^^^^^^^^^^^^^^^^^^^ this is a struct enum variant
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct Variant<F: Form = MetaForm> {
	/// The name of the struct variant.
	name: &'static str,
	/// The fields of the struct variant.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	fields: Vec<Field<F>>,
	/// The discriminant of the variant.
	///
	/// # Note
	///
	/// Even though setting the discriminant is optional
	/// every C-like enum variant has a discriminant specified
	/// upon compile-time.
	#[serde(skip_serializing_if = "Option::is_none")]
	discriminant: Option<u64>,
}

impl IntoCompact for Variant {
	type Output = Variant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		Variant {
			name: self.name,
			fields: registry.map_into_compact(self.fields),
			discriminant: self.discriminant,
		}
	}
}

impl Variant {
	/// Creates a new variant with the given fields.
	pub fn with_fields<F>(name: &'static str, fields: FieldsBuilder<F>) -> Self {
		Self {
			name,
			fields: fields.done(),
			discriminant: None,
		}
	}

	/// Creates a new variant with the given discriminant.
	pub fn with_discriminant(name: &'static str, discriminant: u64) -> Self {
		Self {
			name,
			fields: Vec::new(),
			discriminant: Some(discriminant),
		}
	}
}
