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
	form::{Form, FreeForm, CompactForm},
	HasTypeId,
	TypeId,
	Registry,
	IntoCompact,
	IntoCompactError,
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
pub struct TypeDef<F: Form = FreeForm> {
	/// Stores count and names of all generic parameters.
	///
	/// This can be used to verify that type id's refer to
	/// correct instantiations of a generic type.
	generic_params: GenericParams<F>,
	/// The underlying structure of the type definition.
	kind: TypeDefKind<F>,
}

impl IntoCompact for TypeDef<FreeForm> {
	type Output = TypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(TypeDef {
			generic_params: self.generic_params.into_compact(registry)?,
			kind: self.kind.into_compact(registry)?,
		})
	}
}

impl TypeDef<FreeForm> {
	pub fn new<G, K>(generic_params: G, kind: K) -> Self
	where
		G: IntoIterator<Item = <FreeForm as Form>::String>,
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
pub struct GenericParams<F: Form = FreeForm> {
	params: Vec<GenericArg<F>>,
}

impl IntoCompact for GenericParams<FreeForm> {
	type Output = GenericParams<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(GenericParams {
			params: self.params
				.into_iter()
				.map(|param| param.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?
		})
	}
}

impl GenericParams {
	pub fn empty() -> Self {
		Self { params: vec![] }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GenericArg<F: Form = FreeForm> {
	name: F::String,
}

impl IntoCompact for GenericArg<FreeForm> {
	type Output = GenericArg<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(GenericArg {
			name: registry.resolve_string(self.name)?
		})
	}
}

impl From<<FreeForm as Form>::String> for GenericArg {
	fn from(name: <FreeForm as Form>::String) -> Self {
		Self { name }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
pub enum TypeDefKind<F: Form = FreeForm> {
	Builtin,
	Struct(TypeDefStruct<F>),
	TupleStruct(TypeDefTupleStruct<F>),
	ClikeEnum(TypeDefClikeEnum<F>),
	Enum(TypeDefEnum<F>),
	Union(TypeDefUnion<F>),
}

impl IntoCompact for TypeDefKind<FreeForm> {
	type Output = TypeDefKind<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		match self {
			TypeDefKind::Builtin => Ok(TypeDefKind::Builtin),
			TypeDefKind::Struct(r#struct) => r#struct.into_compact(registry).map(Into::into),
			TypeDefKind::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).map(Into::into),
			TypeDefKind::ClikeEnum(clike_enum) => clike_enum.into_compact(registry).map(Into::into),
			TypeDefKind::Enum(r#enum) => r#enum.into_compact(registry).map(Into::into),
			TypeDefKind::Union(union) => union.into_compact(registry).map(Into::into),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefStruct<F: Form = FreeForm> {
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for TypeDefStruct<FreeForm> {
	type Output = TypeDefStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(TypeDefStruct {
			fields: self.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?
		})
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
pub struct NamedField<F: Form = FreeForm> {
	name: F::String,
	#[serde(rename = "type")]
	ty: F::TypeId,
}

impl IntoCompact for NamedField<FreeForm> {
	type Output = NamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(NamedField {
			name: registry.resolve_string(self.name)?,
			ty: registry.resolve_type_id(&self.ty)?,
		})
	}
}

impl NamedField {
	pub fn new<T>(name: <FreeForm as Form>::String, ty: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self { name, ty: ty.into() }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefTupleStruct<F: Form = FreeForm> {
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for TypeDefTupleStruct<FreeForm> {
	type Output = TypeDefTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(TypeDefTupleStruct {
			fields: self.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?
		})
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct UnnamedField<F: Form = FreeForm> {
	#[serde(rename = "type")]
	ty: F::TypeId,
}

impl IntoCompact for UnnamedField<FreeForm> {
	type Output = UnnamedField<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(UnnamedField {
			ty: registry.resolve_type_id(&self.ty)?,
		})
	}
}

impl UnnamedField {
	pub fn new<T>() -> Self
	where
		T: HasTypeId,
	{
		Self { ty: T::type_id() }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefClikeEnum<F: Form = FreeForm> {
	variants: Vec<ClikeEnumVariant<F>>,
}

impl IntoCompact for TypeDefClikeEnum<FreeForm> {
	type Output = TypeDefClikeEnum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(TypeDefClikeEnum {
			variants: self.variants
				.into_iter()
				.map(|variant| variant.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?,
		})
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct ClikeEnumVariant<F: Form = FreeForm> {
	name: F::String,
	discriminant: u64,
}

impl IntoCompact for ClikeEnumVariant<FreeForm> {
	type Output = ClikeEnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(ClikeEnumVariant {
			name: registry.resolve_string(self.name)?,
			discriminant: self.discriminant,
		})
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefEnum<F: Form = FreeForm> {
	variants: Vec<EnumVariant<F>>,
}

impl IntoCompact for TypeDefEnum<FreeForm> {
	type Output = TypeDefEnum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(TypeDefEnum {
			variants: self.variants
				.into_iter()
				.map(|variant| variant.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?,
		})
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
pub enum EnumVariant<F: Form = FreeForm> {
	Unit(EnumVariantUnit<F>),
	Struct(EnumVariantStruct<F>),
	TupleStruct(EnumVariantTupleStruct<F>),
}

impl IntoCompact for EnumVariant<FreeForm> {
	type Output = EnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		match self {
			EnumVariant::Unit(unit) => unit.into_compact(registry).map(Into::into),
			EnumVariant::Struct(r#struct) => r#struct.into_compact(registry).map(Into::into),
			EnumVariant::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).map(Into::into),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantUnit<F: Form = FreeForm> {
	name: F::String,
}

impl IntoCompact for EnumVariantUnit<FreeForm> {
	type Output = EnumVariantUnit<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(EnumVariantUnit {
			name: registry.resolve_string(self.name)?,
		})
	}
}

impl EnumVariantUnit {
	pub fn new(name: &'static str) -> Self {
		Self { name }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantStruct<F: Form = FreeForm> {
	name: F::String,
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for EnumVariantStruct<FreeForm> {
	type Output = EnumVariantStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(EnumVariantStruct {
			name: registry.resolve_string(self.name)?,
			fields: self.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?,
		})
	}
}

impl EnumVariantStruct {
	pub fn new<F>(name: <FreeForm as Form>::String, fields: F) -> Self
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
pub struct EnumVariantTupleStruct<F: Form = FreeForm> {
	name: F::String,
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for EnumVariantTupleStruct<FreeForm> {
	type Output = EnumVariantTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(EnumVariantTupleStruct {
			name: registry.resolve_string(self.name)?,
			fields: self.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?,
		})
	}
}

impl EnumVariantTupleStruct {
	pub fn new<F>(name: <FreeForm as Form>::String, fields: F) -> Self
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
pub struct TypeDefUnion<F: Form = FreeForm> {
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for TypeDefUnion<FreeForm> {
	type Output = TypeDefUnion<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Result<Self::Output, IntoCompactError> {
		Ok(TypeDefUnion {
			fields: self.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Result<Vec<_>, _>>()?,
		})
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
