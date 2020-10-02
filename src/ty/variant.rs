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
use scale::{Decode, Encode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From, Serialize, Deserialize, Encode, Decode)]
#[serde(bound(
	serialize = "T::Type: Serialize, T::String: Serialize",
	deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned"
))]
#[serde(rename_all = "lowercase")]
pub struct TypeDefVariant<T: Form = MetaForm> {
	/// The variants of a variant type
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	variants: Vec<Variant<T>>,
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

impl<T> TypeDefVariant<T>
where
	T: Form
{
	/// Returns the variants of a variant type
	pub fn variants(&self) -> &[Variant<T>] {
		&self.variants
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
#[serde(bound(
	serialize = "T::Type: Serialize, T::String: Serialize",
	deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned"
))]
pub struct Variant<T: Form = MetaForm> {
	/// The name of the variant.
	name: T::String,
	/// The fields of the variant.
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	fields: Vec<Field<T>>,
	/// The discriminant of the variant.
	///
	/// # Note
	///
	/// Even though setting the discriminant is optional
	/// every C-like enum variant has a discriminant specified
	/// upon compile-time.
	#[serde(skip_serializing_if = "Option::is_none", default)]
	discriminant: Option<u64>,
}

impl IntoCompact for Variant {
	type Output = Variant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		Variant {
			name: self.name.into_compact(registry),
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

impl<T> TypeDefVariant<T>
	where
		T: Form
{
	/// Returns the name of the variant
	pub fn name(&self) -> &T::String {
		&self.name
	}

	/// Returns the fields of the struct variant.
	pub fn fields(&self) -> &[Field<T>] {
		&self.fields
	}

	/// Returns the discriminant of the variant.
	pub fn discriminant(&self) -> Option<u64> {
		self.discriminant
	}
}
