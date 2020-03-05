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
	IntoCompact, Field, Registry,
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
pub struct TypeEnum<F: Form = MetaForm> {
	variants: EnumVariant<F>,
}

impl IntoCompact for TypeEnum {
	type Output = TypeSum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		registry.register_types(self.variants)
	}
}

/// A Rust enum variant.
///
/// This can either be a unit struct, just like in C-like enums,
/// a tuple-struct with unnamed fields,
/// or a struct with named fields.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum EnumVariant<F: Form = MetaForm> {
	/// A unit struct variant.
	Unit(EnumVariantUnit<F>),
	/// A struct variant with fields (either named or unnamed).
	Struct(EnumVariantStruct<F>),
}

impl IntoCompact for EnumVariant {
	type Output = EnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			EnumVariant::Unit(unit) => unit.into_compact(registry).into(),
			EnumVariant::Struct(r#struct) => r#struct.into_compact(registry).into(),
		}
	}
}

/// An unit struct enum variant.
///
/// These are similar to the variants in C-like enums.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
/// //  ^^^^ this is a unit struct enum variant
///     Add(i32, i32),
///     Minus { source: i32 }
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct EnumVariantUnit<F: Form = MetaForm> {
	/// The name of the variant.
	name: F::String,
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

impl IntoCompact for EnumVariantUnit {
	type Output = EnumVariantUnit<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantUnit {
			name: registry.register_string(self.name),
			discriminant: self.discriminant,
		}
	}
}

impl EnumVariantUnit {
	/// Creates a new unit struct variant.
	pub fn new(name: &'static str, discriminant: Option<u64>) -> Self {
		Self { name, discriminant }
	}
}

/// A struct enum variant with either named (struct) or unnamed (tuple struct) fields.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
///     Add(i32, i32),
/// //  ^^^^^^^^^^^^^ this is a tuple-struct enum variant
///     Minus { source: i32 }
/// //  ^^^^^^^^^^^^^^^^^^^^^ this is a struct enum variant
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct EnumVariantStruct<F: Form = MetaForm> {
	/// The name of the struct variant.
	name: F::String,
	/// The fields of the struct variant.
	fields: Vec<Field<F>>,
}

impl IntoCompact for EnumVariantStruct {
	type Output = EnumVariantStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantStruct {
			name: registry.register_string(self.name),
			fields: registry.register_types(self.fields),
		}
	}
}

impl EnumVariantStruct {
	/// Creates a new struct variant from the given fields.
	pub fn new<F>(name: <MetaForm as Form>::String, fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
		}
	}
}
