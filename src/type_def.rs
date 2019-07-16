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

use crate::{
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Metadata, Registry,
};
use derive_more::From;
use serde::Serialize;

/// Types implementing this trait can communicate their type structure.
///
/// If the current type contains any other types, `type_def` would register their metadata into the given
/// `registry`. For instance, `<Option<MyStruct>>::type_def()` would register `MyStruct` metadata. All
/// implementation must register these contained types' metadata.
pub trait HasTypeDef {
	fn type_def() -> TypeDef;
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDef<F: Form = MetaForm> {
	/// Stores count and names of all generic parameters.
	///
	/// This can be used to verify that type id's refer to
	/// correct instantiations of a generic type.
	generic_params: GenericParams<F>,
	/// The underlying structure of the type definition.
	kind: TypeDefKind<F>,
}

impl IntoCompact for TypeDef {
	type Output = TypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeDef {
			generic_params: self.generic_params.into_compact(registry),
			kind: self.kind.into_compact(registry),
		}
	}
}

impl TypeDef {
	pub fn new<G, K>(generic_params: G, kind: K) -> Self
	where
		G: IntoIterator<Item = <MetaForm as Form>::String>,
		K: Into<TypeDefKind>,
	{
		Self {
			generic_params: generic_params
				.into_iter()
				.map(|name| GenericArg::from(name))
				.collect::<Vec<_>>()
				.into(),
			kind: kind.into(),
		}
	}
}

impl<K> From<K> for TypeDef
where
	K: Into<TypeDefKind>,
{
	fn from(kind: K) -> Self {
		Self {
			generic_params: GenericParams::empty(),
			kind: kind.into(),
		}
	}
}

impl TypeDef {
	pub fn builtin() -> Self {
		Self {
			generic_params: GenericParams::empty(),
			kind: TypeDefKind::Builtin,
		}
	}

	pub fn kind(&self) -> &TypeDefKind {
		&self.kind
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct GenericParams<F: Form = MetaForm> {
	params: Vec<GenericArg<F>>,
}

impl IntoCompact for GenericParams {
	type Output = GenericParams<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		GenericParams {
			params: self
				.params
				.into_iter()
				.map(|param| param.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl GenericParams {
	pub fn empty() -> Self {
		Self { params: vec![] }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GenericArg<F: Form = MetaForm> {
	name: F::String,
}

impl IntoCompact for GenericArg {
	type Output = GenericArg<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		GenericArg {
			name: registry.register_string(self.name),
		}
	}
}

impl From<<MetaForm as Form>::String> for GenericArg {
	fn from(name: <MetaForm as Form>::String) -> Self {
		Self { name }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
pub enum TypeDefKind<F: Form = MetaForm> {
	Builtin,
	Struct(TypeDefStruct<F>),
	TupleStruct(TypeDefTupleStruct<F>),
	ClikeEnum(TypeDefClikeEnum<F>),
	Enum(TypeDefEnum<F>),
	Union(TypeDefUnion<F>),
}

impl IntoCompact for TypeDefKind {
	type Output = TypeDefKind<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeDefKind::Builtin => TypeDefKind::Builtin,
			TypeDefKind::Struct(r#struct) => r#struct.into_compact(registry).into(),
			TypeDefKind::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).into(),
			TypeDefKind::ClikeEnum(clike_enum) => clike_enum.into_compact(registry).into(),
			TypeDefKind::Enum(r#enum) => r#enum.into_compact(registry).into(),
			TypeDefKind::Union(union) => union.into_compact(registry).into(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefStruct<F: Form = MetaForm> {
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
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct NamedField<F: Form = MetaForm> {
	name: F::String,
	#[serde(rename = "type")]
	ty: F::TypeId,
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
	pub fn new(name: <MetaForm as Form>::String, ty: MetaType) -> Self {
		Self { name, ty }
	}

	pub fn of<T>(name: <MetaForm as Form>::String) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(name, MetaType::new::<T>())
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefTupleStruct<F: Form = MetaForm> {
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
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = UnnamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}

	pub fn unit() -> Self {
		Self { fields: vec![] }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct UnnamedField<F: Form = MetaForm> {
	#[serde(rename = "type")]
	ty: F::TypeId,
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
	pub fn new(meta_type: MetaType) -> Self {
		Self { ty: meta_type }
	}

	pub fn of<T>() -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new(MetaType::new::<T>())
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefClikeEnum<F: Form = MetaForm> {
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
	pub fn new<V>(variants: V) -> Self
	where
		V: IntoIterator<Item = ClikeEnumVariant>,
	{
		Self {
			variants: variants.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct ClikeEnumVariant<F: Form = MetaForm> {
	name: F::String,
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

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefEnum<F: Form = MetaForm> {
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
	pub fn new<V>(variants: V) -> Self
	where
		V: IntoIterator<Item = EnumVariant>,
	{
		Self {
			variants: variants.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
#[serde(bound = "F::TypeId: Serialize")]
pub enum EnumVariant<F: Form = MetaForm> {
	Unit(EnumVariantUnit<F>),
	Struct(EnumVariantStruct<F>),
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

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantUnit<F: Form = MetaForm> {
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
	pub fn new(name: &'static str) -> Self {
		Self { name }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct EnumVariantStruct<F: Form = MetaForm> {
	name: F::String,
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

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct EnumVariantTupleStruct<F: Form = MetaForm> {
	name: F::String,
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

#[derive(PartialEq, Eq, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct TypeDefUnion<F: Form = MetaForm> {
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
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}
}
