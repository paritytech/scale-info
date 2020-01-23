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
	IntoCompact, MetaType, Metadata, Registry,
};
use derive_more::From;
use serde::Serialize;

/// A type definition represents the internal structure of a concrete type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum TypeDef<F: Form = MetaForm> {
	/// A struct with named fields.
	Struct(TypeDefStruct<F>),
	/// A tuple-struct with unnamed fields.
	TupleStruct(TypeDefTupleStruct<F>),
	/// A C-like enum with simple named variants.
	ClikeEnum(TypeDefClikeEnum<F>),
	/// A Rust enum with different kinds of variants.
	Enum(TypeDefEnum<F>),
	/// An unsafe Rust union type.
	Union(TypeDefUnion<F>),
}

impl IntoCompact for TypeDef {
	type Output = TypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeDef::Struct(r#struct) => r#struct.into_compact(registry).into(),
			TypeDef::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).into(),
			TypeDef::ClikeEnum(clike_enum) => clike_enum.into_compact(registry).into(),
			TypeDef::Enum(r#enum) => r#enum.into_compact(registry).into(),
			TypeDef::Union(union) => union.into_compact(registry).into(),
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
pub struct TypeDefStruct<F: Form = MetaForm> {
	/// The named fields of the struct.
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for TypeDefStruct {
	type Output = TypeDefStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefStruct {
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefStruct {
	/// Creates a new struct definition with named fields.
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}
}

/// A named field.
///
/// This can be a named field of a struct type or a struct variant.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct NamedField<F: Form = MetaForm> {
	/// The name of the field.
	name: F::String,
	/// The type of the field.
	#[serde(rename = "type")]
	ty: F::Type,
}

impl IntoCompact for NamedField {
	type Output = NamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		NamedField {
			name: registry.register_string(self.name),
			ty: registry.register_type(&self.ty),
		}
	}
}

impl NamedField {
	/// Creates a new named field.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(name: <MetaForm as Form>::String, ty: MetaType) -> Self {
		Self { name, ty }
	}

	/// Creates a new named field.
	///
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn of<T>(name: <MetaForm as Form>::String) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(name, MetaType::new::<T>())
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
pub struct TypeDefTupleStruct<F: Form = MetaForm> {
	/// The unnamed fields.
	#[serde(rename = "types")]
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for TypeDefTupleStruct {
	type Output = TypeDefTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefTupleStruct {
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefTupleStruct {
	/// Creates a new tuple-struct.
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = UnnamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}

	/// Creates the unit tuple-struct that has no fields.
	pub fn unit() -> Self {
		Self { fields: vec![] }
	}
}

/// An unnamed field from either a tuple-struct type or a tuple-struct variant.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
#[serde(transparent)]
pub struct UnnamedField<F: Form = MetaForm> {
	/// The type of the unnamed field.
	#[serde(rename = "type")]
	ty: F::Type,
}

impl IntoCompact for UnnamedField {
	type Output = UnnamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		UnnamedField {
			ty: registry.register_type(&self.ty),
		}
	}
}

impl UnnamedField {
	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(meta_type: MetaType) -> Self {
		Self { ty: meta_type }
	}

	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate from a given compile-time type.
	pub fn of<T>() -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(MetaType::new::<T>())
	}
}

/// A C-like enum type.
///
/// # Example
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
/// or an empty enum (for marker purposes)
/// ```
/// enum JustAMarker {}
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDefClikeEnum<F: Form = MetaForm> {
	/// The variants of the C-like enum.
	#[serde(rename = "variants")]
	variants: Vec<ClikeEnumVariant<F>>,
}

impl IntoCompact for TypeDefClikeEnum {
	type Output = TypeDefClikeEnum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefClikeEnum {
			variants: self
				.variants
				.into_iter()
				.map(|variant| variant.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefClikeEnum {
	/// Creates a new C-like enum from the given variants.
	pub fn new<V>(variants: V) -> Self
	where
		V: IntoIterator<Item = ClikeEnumVariant>,
	{
		Self {
			variants: variants.into_iter().collect(),
		}
	}
}

/// A C-like enum variant.
///
/// # Example
///
/// ```
/// enum Food {
///     Pizza,
/// //  ^^^^^ this is a C-like enum variant
///     Salad = 1337,
/// //  ^^^^^ this as well
///     Apple,
/// //  ^^^^^ and this
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct ClikeEnumVariant<F: Form = MetaForm> {
	/// The name of the variant.
	name: F::String,
	/// The discriminant of the variant.
	///
	/// # Note
	///
	/// Even though setting the discriminant is optional
	/// every C-like enum variant has a discriminant specified
	/// upon compile-time.
	discriminant: u64,
}

impl IntoCompact for ClikeEnumVariant {
	type Output = ClikeEnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		ClikeEnumVariant {
			name: registry.register_string(self.name),
			discriminant: self.discriminant,
		}
	}
}

impl ClikeEnumVariant {
	/// Creates a new C-like enum variant.
	pub fn new<D>(name: <MetaForm as Form>::String, discriminant: D) -> Self
	where
		D: Into<u64>,
	{
		Self {
			name,
			discriminant: discriminant.into(),
		}
	}
}

/// A Rust enum, aka tagged union.
///
/// # Examples
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDefEnum<F: Form = MetaForm> {
	/// The variants of the enum.
	variants: Vec<EnumVariant<F>>,
}

impl IntoCompact for TypeDefEnum {
	type Output = TypeDefEnum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefEnum {
			variants: self
				.variants
				.into_iter()
				.map(|variant| variant.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefEnum {
	/// Creates a new Rust enum from the given variants.
	pub fn new<V>(variants: V) -> Self
	where
		V: IntoIterator<Item = EnumVariant>,
	{
		Self {
			variants: variants.into_iter().collect(),
		}
	}
}

/// A Rust enum variant.
///
/// This can either be a unit struct, just like in C-like enums,
/// a tuple-struct with unnamed fields,
/// or a struct with named fields.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum EnumVariant<F: Form = MetaForm> {
	/// A unit struct variant.
	Unit(EnumVariantUnit<F>),
	/// A struct variant with named fields.
	Struct(EnumVariantStruct<F>),
	/// A tuple-struct variant with unnamed fields.
	TupleStruct(EnumVariantTupleStruct<F>),
}

impl IntoCompact for EnumVariant {
	type Output = EnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			EnumVariant::Unit(unit) => unit.into_compact(registry).into(),
			EnumVariant::Struct(r#struct) => r#struct.into_compact(registry).into(),
			EnumVariant::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).into(),
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
}

impl IntoCompact for EnumVariantUnit {
	type Output = EnumVariantUnit<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantUnit {
			name: registry.register_string(self.name),
		}
	}
}

impl EnumVariantUnit {
	/// Creates a new unit struct variant.
	pub fn new(name: &'static str) -> Self {
		Self { name }
	}
}

/// A struct enum variant with named fields.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
///     Add(i32, i32),
///     Minus { source: i32 }
/// //  ^^^^^^^^^^^^^^^^^^^^^ this is a struct enum variant
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct EnumVariantStruct<F: Form = MetaForm> {
	/// The name of the struct variant.
	name: F::String,
	/// The fields of the struct variant.
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for EnumVariantStruct {
	type Output = EnumVariantStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantStruct {
			name: registry.register_string(self.name),
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
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

/// A tuple struct enum variant.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
///     Add(i32, i32),
/// //  ^^^^^^^^^^^^^ this is a tuple-struct enum variant
///     Minus {
///         source: i32,
///     }
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct EnumVariantTupleStruct<F: Form = MetaForm> {
	/// The name of the variant.
	name: F::String,
	/// The fields of the variant.
	#[serde(rename = "types")]
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for EnumVariantTupleStruct {
	type Output = EnumVariantTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantTupleStruct {
			name: registry.register_string(self.name),
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl EnumVariantTupleStruct {
	/// Creates a new tuple struct enum variant from the given fields.
	pub fn new<F>(name: <MetaForm as Form>::String, fields: F) -> Self
	where
		F: IntoIterator<Item = UnnamedField>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
		}
	}
}

/// A union, aka untagged union, type definition.
///
/// # Example
///
/// ```
/// union SmallVecI32 {
///     inl: [i32; 8],
///     ext: *mut i32,
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeDefUnion<F: Form = MetaForm> {
	/// The fields of the union.
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for TypeDefUnion {
	type Output = TypeDefUnion<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDefUnion {
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeDefUnion {
	/// Creates a new union type definition from the given named fields.
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}
}
