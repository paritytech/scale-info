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

use crate::{form::{CompactForm, Form, MetaForm}, IntoCompact, Registry, MetaType, Namespace, Path, Field, NoFields, Fields, PathBuilder };
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
	path: Path<F>,
	variants: Vec<Variant<F>>,
}

impl IntoCompact for TypeVariant {
	type Output = TypeVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeVariant {
			path: self.path.into_compact(registry),
			variants: registry.map_into_compact(self.variants),
		}
	}
}

impl TypeVariant {
	pub fn new(name: &'static str, namespace: Namespace) -> TypeVariantBuilder {
		TypeVariantBuilder {
			path: Path::new(name, namespace)
		}
	}
}

pub struct TypeVariantBuilder {
	path: PathBuilder
}

impl TypeVariantBuilder {
	pub fn type_params<I>(self, type_params: I) -> Self
	where
		I: IntoIterator<Item = MetaType>
	{
		let mut this = self;
		this.path.type_params(type_params);
		this
	}

	pub fn variants<F>(self, variants: Variants<F>) -> TypeVariant {
		TypeVariant { path: self.path.done(), variants: variants.variants() }
	}
}

/// A struct enum variant with either named (struct) or unnamed (tuple struct) fields.
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
	pub fn with_fields<F>(name: <MetaForm as Form>::String, fields: F) -> Self
	where
		F: IntoIterator<Item = Field>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
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
/// Build a type where *any* variants consist of fields.
pub enum VariantFields {}
/// Build a type where *all* variants have no fields and a discriminant (e.g. a Clike enum)
pub enum Discriminant {}

pub struct Variants<T = NoVariants> {
	variants: Vec<Variant>,
	marker: PhantomData<fn() -> T>,
}

impl Variants {
	pub fn new<T>() -> Variants<T> {
		Variants::<T> { variants: Vec::new(), marker: Default::default() }
	}

	pub fn with_fields() -> Variants<VariantFields> {
		Self::new::<VariantFields>()
	}

	pub fn with_discriminants() -> Variants<Discriminant> {
		Self::new::<Discriminant>()
	}
}

impl<T> Variants<T> {
	pub fn variants(self) -> Vec<Variant> {
		self.variants
	}
}

impl Variants<VariantFields> {
	pub fn variant<F>(self, name: <MetaForm as Form>::String, fields: Fields<F>) -> Self {
		let mut this = self;
		this.variants.push(Variant::with_fields(name, fields));
		this
	}

	pub fn variant_unit(self, name: <MetaForm as Form>::String) -> Self {
		self.variant::<NoFields>(name, Fields::new::<NoFields>())
	}
}

impl Variants<Discriminant> {
	pub fn variant(self, name: <MetaForm as Form>::String, discriminant: u64) -> Self {
		let mut this = self;
		this.variants.push(Variant::with_discriminant(name, discriminant));
		this
	}
}




