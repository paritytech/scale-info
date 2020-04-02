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
	form::{CompactForm, Form, MetaForm},
	CompletePath, Field, Fields, FieldsBuilder, IntoCompact, MetaType, NoFields, Path, PathBuilder, Registry,
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
pub struct TypeVariant<F: Form = MetaForm> {
	#[serde(skip_serializing_if = "Path::is_empty")]
	path: Path<F>,
	/// The generic type parameters of the type in use.
	#[serde(rename = "params", skip_serializing_if = "Vec::is_empty")]
	type_params: Vec<F::TypeId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	variants: Vec<Variant<F>>,
}

impl IntoCompact for TypeVariant {
	type Output = TypeVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeVariant {
			path: self.path.into_compact(registry),
			type_params: registry.register_types(self.type_params),
			variants: registry.map_into_compact(self.variants),
		}
	}
}

impl TypeVariant {
	#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_ret_no_self))]
	pub fn new() -> TypeVariantBuilder {
		TypeVariantBuilder::default()
	}
}

#[derive(Default)]
pub struct TypeVariantBuilder {
	path: Path,
	type_params: Vec<MetaType>,
}

impl TypeVariantBuilder {
	/// Set the Path for the type
	///
	/// # Panics
	///
	/// If the Path is invalid
	pub fn path(self, path: PathBuilder<CompletePath>) -> Self {
		let mut this = self;
		this.path = path.done().expect("Should be a valid path");
		this
	}

	pub fn type_params<I>(self, type_params: I) -> Self
	where
		I: IntoIterator<Item = MetaType>,
	{
		let mut this = self;
		this.type_params = type_params.into_iter().collect();
		this
	}

	pub fn variants<F>(self, variants: VariantsBuilder<F>) -> TypeVariant {
		TypeVariant {
			path: self.path,
			type_params: self.type_params,
			variants: variants.done(),
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
	name: F::String,
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
			name: registry.register_string(self.name),
			fields: registry.map_into_compact(self.fields),
			discriminant: self.discriminant,
		}
	}
}

impl Variant {
	/// Creates a new variant with the given fields.
	pub fn with_fields<F>(name: <MetaForm as Form>::String, fields: FieldsBuilder<F>) -> Self {
		Self {
			name,
			fields: fields.done(),
			discriminant: None,
		}
	}

	/// Creates a new variant with the given discriminant.
	pub fn with_discriminant(name: <MetaForm as Form>::String, discriminant: u64) -> Self {
		Self {
			name,
			fields: Vec::new(),
			discriminant: Some(discriminant),
		}
	}
}

/// Build a type with no variants.
pub enum NoVariants {}
/// Build a type where at least one variant has fields.
pub enum VariantFields {}
/// Build a type where *all* variants have no fields and a discriminant (e.g. a
/// Clike enum)
pub enum Discriminant {}

/// Empty enum for VariantsBuilder constructors for the type builder DSL.
pub enum Variants {}

impl Variants {
	/// Build a set of variants, at least one of which will have fields.
	pub fn with_fields() -> VariantsBuilder<VariantFields> {
		VariantsBuilder::new()
	}

	/// Build a set of variants, none of which will have fields, but all of
	/// which will have discriminants.
	pub fn with_discriminants() -> VariantsBuilder<Discriminant> {
		VariantsBuilder::new()
	}
}

#[derive(Default)]
pub struct VariantsBuilder<T> {
	variants: Vec<Variant>,
	marker: PhantomData<fn() -> T>,
}

impl VariantsBuilder<VariantFields> {
	pub fn variant<F>(self, name: <MetaForm as Form>::String, fields: FieldsBuilder<F>) -> Self {
		let mut this = self;
		this.variants.push(Variant::with_fields(name, fields));
		this
	}

	pub fn variant_unit(self, name: <MetaForm as Form>::String) -> Self {
		self.variant::<NoFields>(name, Fields::unit())
	}
}

impl VariantsBuilder<Discriminant> {
	pub fn variant(self, name: <MetaForm as Form>::String, discriminant: u64) -> Self {
		let mut this = self;
		this.variants.push(Variant::with_discriminant(name, discriminant));
		this
	}
}

impl<T> VariantsBuilder<T> {
	pub fn new() -> Self {
		VariantsBuilder {
			variants: Vec::new(),
			marker: Default::default(),
		}
	}

	pub fn done(self) -> Vec<Variant> {
		self.variants
	}
}
